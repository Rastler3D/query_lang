mod query;





#[cfg(test)]
mod tests {
    use nom::bytes::complete::is_not;
    use crate::query::parser::{field, predicate, string, unescape_string};
    use super::*;

    #[test]
    fn it_works() {
        let a = r#"{
          "h.[19].ello": "world",
          "hel[10].string": {
            "ol": 12
          }
        }
        "#;
        let res = predicate(a);
        println!("{res:#?}");
        let field_str = r#""""#;
        let res = field(field_str);
        println!("{res:#?}");
    }
}
