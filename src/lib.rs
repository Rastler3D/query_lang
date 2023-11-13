mod query;

#[cfg(test)]
mod tests {
    use nom::bytes::complete::tag;
    use nom::character::complete::char;
    use nom::IResult;
    use crate::query::parser::{field, predicate};
    use crate::query::utils::{separated_permutation, separated_tuple};

    #[test]
    fn it_works() {
        let a = r#"{
          "h[19].ello": "world",
          "hel[10].string": {
            "ol": 12
          }
        }
        "#;
        let res = predicate(a);
        println!("{res:#?}");
        let field_str = r#""hel""#;
        let res = field(field_str);
        println!("{res:#?}");

        let res: IResult<&str,_,nom::error::Error<&str>> = separated_tuple(char(','), (tag("Hello"),))("Hello,World");
        println!("{res:#?}");

        let res: IResult<&str,_,nom::error::Error<&str>> = separated_permutation(char(','), (tag("Hello"),tag("World"), char('!')))("Hello,World,!");
        println!("{res:#?}");
    }
}
