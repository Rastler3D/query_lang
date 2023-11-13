use nom::{IResult, Parser};
use nom::error::ParseError;
use nom::sequence::{preceded, terminated, tuple};
use nom::error::ErrorKind;
use nom::Err;


pub fn separated_tuple<I,T: SeparatedTuple<I,O,E>,E,O,O1, S: Parser<I,O1,E>>(mut separator: S, mut tuple: T) -> impl FnMut(I) -> IResult<I,O,E>{
    move |input: I| T::parse(&mut tuple, &mut separator, input)
}
pub trait SeparatedTuple<I, O, E> {
    fn parse<U, S: Parser<I,U,E>>(x: &mut Self, separator: &mut S, input: I) -> IResult<I, O, E>;
}


macro_rules! impl_sep_tuple {
   () => {
        impl<I,Err: ParseError<I>> SeparatedTuple<I,(),Err> for ()
        {
            fn parse<U, S: Parser<I,U,Err>>((): &mut Self, separator: &mut S, input: I) -> IResult<I, (), Err> {
                Ok((input, ()))
            }
        }
   };
    ($t0: ident $(, $tn:ident)* ) => {
        impl_sep_tuple!($($tn),*);

        paste::paste!{
            impl<[<Out $t0>], $([<Out $tn>],)* $t0: Parser<Input,[<Out $t0>],Err>, $($tn: Parser<Input,[<Out $tn>],Err>,)* Input,Err: ParseError<Input>> SeparatedTuple<Input,([<Out $t0>], $([<Out $tn>],)*), Err> for ($t0, $($tn,)*)
            {
                fn parse<U, S: Parser<Input,U,Err>>((ref mut [<$t0:lower>], $(ref mut [<$tn:lower>],)*): &mut Self, mut separator: &mut S, input: Input) -> IResult<Input, ([<Out $t0>], $([<Out $tn>],)*), Err> {
                    let (input, [<$t0:lower>]) = [<$t0:lower>].parse(input)?;
                    $(
                        let (input, _) = separator.parse(input)?;
                        let (input, [<$tn:lower>]) = [<$tn:lower>].parse(input)?;
                    )*

                    Ok((input, ([<$t0:lower>],$( [<$tn:lower>],)*)))
                }
            }
        }
    };
}

impl_sep_tuple!{
    A,B,C,D,E,F,G
}

pub trait SeparatedPermutation<I, O, E> {
    /// Tries to apply all parsers in the tuple in various orders until all of them succeed
    fn permutation<U, S: Parser<I,U,E>>(x: &mut Self, separator: &mut S, input: I) -> IResult<I, O, E>;
}

pub fn separated_permutation<I,T: SeparatedPermutation<I,O,E>,E,O,O1, S: Parser<I,O1,E>>(mut separator: S, mut permutation: T) -> impl FnMut(I) -> IResult<I,O,E>{
    move |input: I| T::permutation(&mut permutation, &mut separator, input)
}

macro_rules! impl_sep_permutation {
    () => {};
    ($t0:ident $(,$tn:ident)* ) => {
        impl_sep_permutation!($($tn),*);
        impl_sep_permutation_inner!($t0 $(,$tn)* );
    };
}

macro_rules! impl_sep_permutation_inner {
   () => {
        impl<I,Error: ParseError<I>> SeparatedPermutation<I,(),Error> for ()
        {
            fn permutation<U, S: Parser<I,U,Error>>((): &mut Self, separator: &mut S, input: I) -> IResult<I, (), Error> {
                Ok((input, ()))
            }
        }
   };
    ($($tn:ident),+ ) => {
        paste::paste!{
            impl<$([<Out $tn>],)+ $($tn: Parser<Input,[<Out $tn>],Error>,)+ Input: Clone,Error: ParseError<Input>> SeparatedPermutation<Input,($([<Out $tn>],)*), Error> for ($($tn,)*)
            {
                fn permutation<U, S: Parser<Input,U,Error>>(($(ref mut [<$tn:lower>],)*): &mut Self, mut separator: &mut S, mut input: Input) -> IResult<Input, ($([<Out $tn>],)*), Error> {
                    $(
                    let mut [<res_ $tn:lower>] = Option::<[<Out $tn>]>::None;
                    )*
                    let mut first = true;
                    loop {
                        match ($(&[<res_ $tn:lower>]),+) {
                            ($(Some([<res_ $tn:lower>])),+) => break,
                            _  if !first => {
                                (input,_) = separator.parse(input)?;
                            },
                            _ => {
                                first = false;
                            }
                        }
                        let mut err: Option<Error> = None;
                        $(
                        if [<res_ $tn:lower>].is_none() {
                            match [<$tn:lower>].parse(input.clone()) {
                                Ok((i, o)) => {
                                    input = i;
                                    [<res_ $tn:lower>] = Some(o);
                                    continue;
                                }
                                Err(Err::Error(e)) => {
                                    err = Some(match err {
                                        Some(err) => err.or(e),
                                        None => e,
                                    });
                                }
                                Err(e) => return Err(e),
                            };
                        }
                        )*

                      // If we reach here, every iterator has either been applied before,
                      // or errored on the remaining input
                      if let Some(err) = err {
                        // There are remaining parsers, and all errored on the remaining input
                        return Err(Err::Error(Error::append(input, ErrorKind::Permutation, err)));
                      }
                    }
                    Ok((input,($([<res_ $tn:lower>].unwrap(),)+)))
                }
            }
        }
    };
}

impl_sep_permutation!{
    A,B,C,D,E,F,G
}