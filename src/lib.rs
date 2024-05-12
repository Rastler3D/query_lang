#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(iter_order_by)]

use derive_more::From;
use serde_json::Value;
use smallvec::SmallVec;
use smartstring::alias::String;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use hashlink::LinkedHashMap;

pub mod query;

impl From<&Value> for Dynamic {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Dynamic::Null,
            Value::Bool(bool) => Dynamic::from(*bool),
            Value::Number(number) => Dynamic::from(Number::from(number)),
            Value::String(string) => Dynamic::from(smartstring::alias::String::from(string)),
            Value::Array(array) => {
                Dynamic::from(array.iter().map(|x| Dynamic::from(x)).collect::<Vec<_>>())
            }
            Value::Object(object) => {
                let map = object
                    .iter()
                    .map(|(key, value)| (key.into(), Dynamic::from(value)))
                    .collect();
                Dynamic::from(Object::Map(Arc::new(RwLock::new(map))))
            }
        }
    }
}

#[derive(Debug)]
pub enum DynamicError {
    NotAnObject,
    NotAnArray,
    ImmutableObject,
    UnableToWrite,
    UnableTORead,
}

#[derive(Clone, Copy, From)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number{
    pub fn as_f64(&self) -> f64{
        match self {
            Number::Int(int) => *int as f64,
            Number::Float(float) => *float
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Number::Int(int) => {
                write!(f, "Int({})", int)
            }
            Number::Float(float) => {
                write!(f, "Float({})", float)
            }
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(int) => {
                Display::fmt(int, f)
            }
            Number::Float(float) => {
                Display::fmt(float, f)
            }
        }
    }
}

impl From<serde_json::Number> for Number {
    fn from(value: serde_json::Number) -> Self {
        Number::from(&value)
    }
}

impl From<Number> for serde_json::Number {
    fn from(value: Number) -> Self {
        match value {
            Number::Int(int) => serde_json::Number::from(int),
            Number::Float(float) => {
                serde_json::Number::from_f64(float).unwrap_or(serde_json::Number::from(0))
            }
        }
    }
}

impl From<&serde_json::Number> for Number {
    fn from(value: &serde_json::Number) -> Self {
        if let Some(x) = value.as_f64() {
            Self::Float(x)
        } else if let Some(x) = value.as_i64() {
            Self::Int(x)
        } else {
            Self::Int(value.as_u64().unwrap_or_default() as i64)
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::Float(first), Number::Int(second))
            | (Number::Int(second), Number::Float(first)) => first.eq(&(*second as f64)),
            (Number::Int(first), Number::Int(second)) => first.eq(second),
            (Number::Float(first), Number::Float(second)) => first.eq(second),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Number::Float(first), Number::Int(second)) => first.partial_cmp(&(*second as f64)),
            (Number::Int(first), Number::Float(second)) => (*first as f64).partial_cmp(second),
            (Number::Int(first), Number::Int(second)) => first.partial_cmp(second),
            (Number::Float(first), Number::Float(second)) => first.partial_cmp(second),
        }
    }
}

#[derive(Clone)]
pub enum Object {
    Map(Arc<RwLock<LinkedHashMap<String, Dynamic>>>),
    DynamicObject(Arc<dyn DynamicObject>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Map(map), Object::Map(other_map)) => {
                let Ok(map) = map.read() else { return false };
                let Ok(other_map) = other_map.read() else {
                    return false;
                };
                map.eq(&other_map)
            }
            (Object::DynamicObject(object), Object::DynamicObject(other_object)) => {
                let object_fields = object.field_values();
                let other_object_fields = other_object.field_values();
                object_fields.eq(other_object_fields)
            }
            (Object::DynamicObject(dyn_object), Object::Map(map_object))
            | (Object::Map(map_object), Object::DynamicObject(dyn_object)) => {
                let Ok(map_object) = map_object.read() else {
                    return false;
                };
                let dyn_object_fields = dyn_object.field_values();

                map_object
                    .iter()
                    .eq_by(dyn_object_fields, |x, y| x.0.eq(y.0) && x.1.eq(&y.1))
            }
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Map(map), Object::Map(other_map)) => {
                let Ok(map) = map.read() else { return None };
                let Ok(other_map) = other_map.read() else {
                    return None;
                };
                map.iter().partial_cmp(other_map.iter())
            }
            (Object::DynamicObject(object), Object::DynamicObject(other_object)) => {
                let object_fields = object
                    .fields()
                    .iter()
                    .map(|&key| other_object.get_field(key).map(|value| (key, value)))
                    .flatten();
                let other_object_fields = other_object
                    .fields()
                    .iter()
                    .map(|&key| other_object.get_field(key).map(|value| (key, value)))
                    .flatten();
                object_fields.partial_cmp(other_object_fields)
            }
            (Object::DynamicObject(dyn_object), Object::Map(map_object)) => {
                let Ok(map_object) = map_object.read() else {
                    return None;
                };
                let dyn_object_fields = dyn_object.field_values();
                dyn_object_fields.partial_cmp_by(map_object.iter(), |x, y| {
                    (x.0, &x.1).partial_cmp(&(y.0, y.1))
                })
            }
            (Object::Map(map_object), Object::DynamicObject(dyn_object)) => {
                let Ok(map_object) = map_object.read() else {
                    return None;
                };
                let dyn_object_fields = dyn_object.field_values();
                map_object.iter()
                    .partial_cmp_by(dyn_object_fields, |x, y| {
                        (x.0.deref(), x.1).partial_cmp(&(y.0, &y.1))
                    })
            }
        }
    }
}

impl From<LinkedHashMap<String, Dynamic>> for Object {
    fn from(value: LinkedHashMap<String, Dynamic>) -> Self {
        Object::Map(Arc::new(RwLock::new(value)))
    }
}

impl<T> From<T> for Object
where
    T: DynamicObject + 'static,
{
    fn from(value: T) -> Self {
        Object::DynamicObject(Arc::new(value))
    }
}
impl Object {
    fn get(&self, key: &str) -> Option<Dynamic> {
        match self {
            Object::Map(map) => map.read().ok()?.get(key).cloned(),
            Object::DynamicObject(object) => object.get_field(key.borrow()),
        }
    }

    fn set(
        &self,
        key: impl Into<String>,
        value: impl Into<Dynamic>,
    ) -> Result<Option<Dynamic>, DynamicError> {
        match self {
            Object::Map(map) => Ok(map
                .write()
                .map_err(|_| DynamicError::UnableToWrite)?
                .insert(key.into(), value.into())),
            Object::DynamicObject(_) => Err(DynamicError::ImmutableObject),
        }
    }

    fn remove(&self, key: &str) -> Result<Option<Dynamic>, DynamicError> {
        match self {
            Object::Map(map) => Ok(map
                .write()
                .map_err(|_| DynamicError::UnableToWrite)?
                .remove(key)),
            Object::DynamicObject(_) => Err(DynamicError::ImmutableObject),
        }
    }
}



#[derive(Clone)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Number(Number),
    String(Arc<String>),
    Array(Arc<RwLock<SmallVec<Dynamic, 10>>>),
    Object(Object),
}

impl PartialEq for Dynamic {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Dynamic::Null, Dynamic::Null) => true,
            (Dynamic::Number(number), Dynamic::Number(other_number)) => number.eq(other_number),
            (Dynamic::Bool(bool), Dynamic::Bool(other_bool)) => bool.eq(other_bool),
            (Dynamic::String(string), Dynamic::String(other_string)) => string.eq(other_string),
            (Dynamic::Array(array), Dynamic::Array(other_array)) => {
                let Ok(array) = array.read() else {
                    return false;
                };
                let Ok(other_array) = other_array.read() else {
                    return false;
                };
                array.eq(other_array.deref())
            }
            (Dynamic::Object(object), Dynamic::Object(other_object)) => object.eq(other_object),
            _ => false,
        }
    }
}

impl PartialOrd for Dynamic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Dynamic::Null, Dynamic::Null) => Some(Ordering::Equal),
            (Dynamic::Number(number), Dynamic::Number(other_number)) => {
                number.partial_cmp(other_number)
            }
            (Dynamic::Bool(bool), Dynamic::Bool(other_bool)) => bool.partial_cmp(other_bool),
            (Dynamic::String(string), Dynamic::String(other_string)) => {
                string.partial_cmp(other_string)
            }
            (Dynamic::Array(array), Dynamic::Array(other_array)) => {
                let Ok(array) = array.read() else { return None };
                let Ok(other_array) = other_array.read() else {
                    return None;
                };
                array.partial_cmp(&other_array)
            }
            (Dynamic::Object(object), Dynamic::Object(other_object)) => {
                object.partial_cmp(other_object)
            }
            (x, y) => x.comparison_order().partial_cmp(&y.comparison_order()),
        }
    }
}

impl Dynamic {
    fn comparison_order(&self) -> u8 {
        match self {
            Dynamic::Null => 1,
            Dynamic::Number(_) => 2,
            Dynamic::String(_) => 3,
            Dynamic::Object(_) => 4,
            Dynamic::Array(_) => 5,
            Dynamic::Bool(_) => 6,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Dynamic::Null)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Dynamic::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Dynamic::Number(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Dynamic::String(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Dynamic::Array(_))
    }
    pub fn is_object(&self) -> bool {
        matches!(self, Dynamic::Object(_))
    }

    pub fn as_null(&self) -> Option<()> {
        if let Dynamic::Null = self {
            return Some(());
        }
        None
    }
    pub fn as_bool(&self) -> Option<&bool> {
        if let Dynamic::Bool(bool) = self {
            return Some(bool);
        }
        None
    }
    pub fn as_number(&self) -> Option<&Number> {
        if let Dynamic::Number(number) = self {
            return Some(number);
        }
        None
    }
    pub fn as_string(&self) -> Option<&String> {
        if let Dynamic::String(string) = self {
            return Some(string);
        }
        None
    }
    pub fn as_array(&self) -> Option<&RwLock<SmallVec<Dynamic,10>>> {
        if let Dynamic::Array(array) = self {
            return Some(array.deref());
        }
        None
    }
    pub fn as_object(&self) -> Option<&Object> {
        if let Dynamic::Object(object) = self {
            return Some(object);
        }
        None
    }

    pub fn as_map(&self) -> Option<Arc<RwLock<LinkedHashMap<String,Dynamic>>>>{
        if let Dynamic::Object(Object::Map(map)) = self{
            return Some(map.clone())
        }
        None
    }

    pub fn remove_object_field(&mut self, key: &str) -> Result<Option<Dynamic>, DynamicError> {
        if let Dynamic::Object(object) = self {
            return object.remove(key);
        }
        Err(DynamicError::NotAnObject)
    }

    pub fn get_object_field(&self, key: &str) -> Option<Dynamic> {
        if let Dynamic::Object(object) = self {
            return object.get(key);
        }
        None
    }

    pub fn set_object_field(
        &mut self,
        key: impl Into<String>,
        value: impl Into<Dynamic>,
    ) -> Result<Option<Dynamic>, DynamicError> {
        if let Dynamic::Object(object) = self {
            return object.set(key, value);
        }
        Err(DynamicError::NotAnObject)
    }

    pub fn get_array_item(&self, index: usize) -> Option<Dynamic> {
        if let Dynamic::Array(object) = self {
            return object.read().ok()?.get(index).cloned();
        }
        None
    }
    pub fn push_array_item(&mut self, item: Dynamic) -> Result<(), DynamicError> {
        if let Dynamic::Array(object) = self {
            return Ok(object
                .write()
                .map_err(|_| DynamicError::UnableToWrite)?
                .push(item));
        }
        Err(DynamicError::NotAnArray)
    }
}

impl<T> From<T> for Dynamic
where
    Object: From<T>,
{
    fn from(value: T) -> Self {
        Dynamic::Object(Object::from(value))
    }
}

impl From<Number> for Dynamic {
    fn from(value: Number) -> Self {
        Dynamic::Number(value)
    }
}

impl From<String> for Dynamic {
    fn from(value: String) -> Self {
        Dynamic::String(Arc::new(value))
    }
}

impl From<bool> for Dynamic {
    fn from(value: bool) -> Self {
        Dynamic::Bool(value)
    }
}

impl From<i64> for Dynamic {
    fn from(value: i64) -> Self {
        Dynamic::Number(Number::Int(value))
    }
}

impl From<Vec<Dynamic>> for Dynamic {
    fn from(value: Vec<Dynamic>) -> Self {
        Dynamic::Array(Arc::new(RwLock::new(value.into())))
    }
}

impl From<SmallVec<Dynamic,10>> for Dynamic {
    fn from(value: SmallVec<Dynamic,10>) -> Self {
        Dynamic::Array(Arc::new(RwLock::new(value.into())))
    }
}

type FieldValues<'a> = impl Iterator<Item = (&'a str, Dynamic)>;
pub trait DynamicObject {
    fn get_field(&self, field: &str) -> Option<Dynamic>;
    fn fields(&self) -> &[&str];
}

impl dyn DynamicObject{
    fn field_values(&self) -> FieldValues {
        self.fields()
            .iter()
            .map(|&key| self.get_field(key).map(|value| (key, value)))
            .flatten()
    }
}

impl Debug for dyn DynamicObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.field_values()).finish()
    }
}

impl Debug for Object {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Map(map) => {
                formatter.write_str("Object ")?;
                Debug::fmt(map.read().map_err(|_| fmt::Error)?.deref(), formatter)
            }
            Object::DynamicObject(obj) => {
                formatter.write_str("DynamicObject ")?;
                Debug::fmt(&obj.deref(), formatter)
            }
        }
    }
}

#[derive(Clone)]
pub struct TestObj {
    pub field1: String,
    pub field2: TestObj2,
}
#[derive(Clone)]
pub struct TestObj2 {
    pub field3: i64,
    pub field4: bool,
}

impl DynamicObject for TestObj {
    fn get_field(&self, field: &str) -> Option<Dynamic> {
        match field {
            "field1" => Some(Dynamic::from(self.field1.clone())),
            "field2" => Some(Dynamic::from(self.field2.clone())),
            _ => None,
        }
    }

    fn fields(&self) -> &[&str] {
        &["field1", "field2"]
    }
}

impl DynamicObject for TestObj2 {
    fn get_field(&self, field: &str) -> Option<Dynamic> {
        match field {
            "field3" => Some(Dynamic::from(self.field3.clone())),
            "field4" => Some(Dynamic::from(self.field4.clone())),
            _ => None,
        }
    }

    fn fields(&self) -> &[&str] {
        &["field3", "field4"]
    }
}
impl Debug for Dynamic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dynamic::Null => formatter.write_str("Null"),
            Dynamic::Bool(boolean) => write!(formatter, "Bool({})", boolean),
            Dynamic::Number(number) => Debug::fmt(number, formatter),
            Dynamic::String(string) => write!(formatter, "String({:?})", string.deref()),
            Dynamic::Array(vec) => {
                formatter.write_str("Array ")?;
                Debug::fmt(vec.read().map_err(|_| fmt::Error)?.deref(), formatter)
            }
            Dynamic::Object(map) => Debug::fmt(map, formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::ast::parser::{expression, script};
    use crate::query::parser::{field, predicate};
    use crate::query::utils::{separated_permutation, separated_tuple};
    use crate::query::{Context, Eval, Script};
    use crate::{Dynamic, TestObj, TestObj2};
    use nom::bytes::complete::tag;
    use nom::character::complete::char;
    use nom::IResult;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn it_works() {
        let a = r#"{
        "result": {
            "$match": {
                "object": "$$CURRENT",
                "predicate":
                    {
                        "field1": "TODWA",
                        "field2.field3": 12,
                        "field2" : {
                            "field4": true
                        }
                    }
                }
            }
        }
        "#;

        let test_struct = TestObj {
            field1: "TODWA".into(),
            field2: TestObj2 {
                field3: 12,
                field4: true,
            },
        };

        let json = json!({
            "field1": "TODWA",
            "field2": {
                "field3": 12,
                "field4": true,
                "field5": [1,2,3,4,5,6,{"field10": 10}]
            }
        });
        let mut script = Script::from_str(a).unwrap();
        let mut context = Context::from([
            ("ROOT", Dynamic::from(&json)),
            ("CURRENT", Dynamic::from(test_struct)),
        ]);
        println!("{:#?}", script.eval_with_context(&mut context));
    }
}
