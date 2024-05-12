use std::str::FromStr;
use serde_json::json;
use query_lang::{Dynamic, TestObj, TestObj2};
use query_lang::query::{Context, Script};

fn main() {
    superluminal_perf::begin_event("my-event");
    let a = r#"{
        "current": "$$CURRENT",
        "array_elem": "$$ROOT.field2.field5[6]",
        "eq": {
            "$eq": [
                "$$ROOT",
                {
                    "field1": "TODWA",
                    "field2": {
                        "field3": 12,
                        "field4": true,
                        "field5": [ 1,2,3,4,5,6, { "field10": 10 } ]
                    }
                }
            ]
        },
        "match": {
            "$match": {
                "object": "$$CURRENT",
                "predicate": {
                    "field1": "TODWA",
                    "field2.field3": 12,
                    "field2": {
                        "field4": true
                    }
                }
            }
        }
    }
    "#;

    let test_struct = TestObj{
        field1: "TODWA".into(),
        field2: TestObj2{
            field3: 12,
            field4: true,
        }
    };

    let json = json!({
            "field1": "TODWA",
            "field2": {
                "field3": 12,
                "field4": true,
                "field5": [1,2,3,4,5,6,{ "field10": 10 }]
            }
        });
    let mut script = Script::from_str(a).unwrap();
    let mut context = Context::from([
        ("ROOT", Dynamic::from(&json)),
        ("CURRENT", Dynamic::from(test_struct))
    ]);
    for i in 0..1000{
        let result = script.eval_with_context(&mut context);
        let res = result.unwrap().as_bool();
    }
    let result = script.eval_with_context(&mut context);
    println!("{:#?}", result);
    superluminal_perf::end_event();
}