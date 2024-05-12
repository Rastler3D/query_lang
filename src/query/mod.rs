use crate::query::ast::expression::{ExprLiteral, Expression};
use crate::query::ast::parser::{parse_predicate, script};
use crate::{Dynamic, DynamicError, Object};
use ahash::RandomState;
use derive_more::From;
use hashlink::LinkedHashMap;
use nom::Finish;
use smartstring::alias::String;
use std::borrow::Borrow;
use std::cell::OnceCell;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::ops::Index;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use hashlink::linked_hash_map::RawEntryMut;
use crate::query::ast::Predicate;

pub mod ast;
mod dynamic_object;
pub mod parser;
pub mod utils;

pub type ParseError = nom::error::Error<std::string::String>;
pub trait Eval {
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError>;
}

#[derive(From, Debug)]
pub struct Script(Expression);

impl Script {
    pub fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let Script(expression) = self;

        expression.eval_with_context(context)
    }
    pub fn eval_with_root(&self, root: Dynamic) -> Result<Dynamic, EvalError> {
        let mut context = Context::new();
        let _ = context.set_variable("ROOT", root);

        self.eval_with_context(&mut context)
    }

    pub fn eval(&self) -> Result<Dynamic, EvalError> {
        let mut context = Context::new();

        self.eval_with_context(&mut context)
    }
}
impl<'a> FromStr for Script {
    type Err = ParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        script(string)
            .map_err(|x| x.to_owned())
            .finish()
            .map(|(_, x)| x)
    }
}

impl<'a> FromStr for Predicate {
    type Err = ParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        parse_predicate(string)
            .map_err(|x| x.to_owned())
            .finish()
            .map(|(_, x)| x)
    }
}

pub struct Context {
    map: Dynamic,
    root: u64,
    current: u64,
}

impl<K, V, const N: usize> From<[(K, V); N]> for Context
where
    K: Into<String>,
    V: Into<Dynamic>,
{
    fn from(values: [(K, V); N]) -> Self {
        let mut context = Context::new();
        for (key, value) in values {
            let _ = context.set_variable(key.into(), value.into());
        }

        context
    }
}

impl Context {
    pub fn new() -> Self {
        let mut map = LinkedHashMap::new();
        map.insert("ROOT".into(), Dynamic::Null);
        map.insert("CURRENT".into(), Dynamic::Null);
        let hasher = map.hasher();
        let root = hasher.hash_one("ROOT");
        let current = hasher.hash_one("CURRENT");
        Context {
            map: Dynamic::Object(Object::from(map)),
            root,
            current,
        }
    }

    pub fn as_dynamic(&self) -> &Dynamic {
        &self.map
    }
    pub fn set_variable(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Dynamic>,
    ) -> Result<Option<Dynamic>, DynamicError> {
        self.map.set_object_field(name, value)
    }

    pub fn set_variable_in_scope<K, V, S, O, E>(
        &mut self,
        key: K,
        value: V,
        scope: S,
    ) -> Result<O, E>
    where
        K: Into<String> + Borrow<str>,
        V: Into<Dynamic>,
        S: Fn(&mut Context) -> Result<O, E>,
        E: From<DynamicError>,
    {
        let prev_variable = self.set_variable(key.borrow(), value)?;
        let result = scope(self);
        if let Some(prev_variable) = prev_variable {
            self.set_variable(key.into(), prev_variable)?;
        } else {
            self.remove_variable(key.borrow())?;
        }
        result
    }
    pub fn get_variable(&self, key: &str) -> Option<Dynamic> {
        self.map.get_object_field(key)
    }

    pub fn get_root(&self) -> Dynamic {
        self.map
            .as_map()
            .and_then(|map| {
                map.read().ok().and_then(|map| {
                    map.raw_entry()
                        .from_hash(self.root, |_| true)
                        .map(|x| x.1.clone())
                })
            })
            .unwrap_or(Dynamic::Null)
    }

    pub fn get_current(&self) -> Dynamic {
        self.map
            .as_map()
            .and_then(|map| {
                map.read().ok().and_then(|map| {
                    map.raw_entry()
                        .from_hash(self.current, |_| true)
                        .map(|x| x.1.clone())
                })
            })
            .unwrap_or(Dynamic::Null)
    }

    pub fn set_root(&self, value: impl Into<Dynamic>) -> Option<Dynamic> {
        self.map
            .as_map()
            .and_then(|map| {
                map.write().ok().and_then(|mut map| {
                    match map.raw_entry_mut()
                        .from_hash(self.root, |_| true)
                    {
                        RawEntryMut::Occupied(mut occupied) => Some(occupied.replace_value(value.into())),
                        RawEntryMut::Vacant(vacant) => {
                            vacant.insert_hashed_nocheck(self.root, "ROOT".into(), value.into());
                            None
                        }
                    }
                })
            })
    }
    pub fn set_current(&self, value: impl Into<Dynamic>) -> Option<Dynamic> {
        self.map
            .as_map()
            .and_then(|map| {
                map.write().ok().and_then(|mut map| {
                    match map.raw_entry_mut()
                        .from_hash(self.current, |_| true)
                    {
                        RawEntryMut::Occupied(mut occupied) => Some(occupied.replace_value(value.into())),
                        RawEntryMut::Vacant(vacant) => {
                            vacant.insert_hashed_nocheck(self.current, "CURRENT".into(), value.into());
                            None
                        }
                    }
                })
            })
    }

    pub fn set_current_in_scope<V, S, O, E>(
        &mut self,
        value: V,
        scope: S,
    ) -> Result<O, E>
        where
            V: Into<Dynamic>,
            S: Fn(&mut Context) -> Result<O, E>,
            E: From<DynamicError>,
    {
        let prev_variable = self.set_current(value).unwrap_or(Dynamic::Null);
        let result = scope(self);
        self.set_current(prev_variable);
        result
    }


    pub fn remove_variable(&mut self, key: &str) -> Result<Option<Dynamic>, DynamicError> {
        self.map.remove_object_field(key)
    }
}

#[derive(Debug, From)]
pub enum EvalError {
    UndefinedVariable,
    DynamicError(DynamicError),
}
