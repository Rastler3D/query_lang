use std::hint::black_box;
use std::str::FromStr;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;
use query_lang::{Dynamic, TestObj, TestObj2};
use query_lang::query::{Context, Script};

pub fn criterion_benchmark(c: &mut Criterion) {
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

    let json = json!({
            "field1": "TODWA",
            "field2": {
                "field3": 12,
                "field4": true,
                "field5": [1,2,3,4,5,6,{ "field10": 10 }]
            }
        });
    let mut script = Script::from_str(a).unwrap();
    let dynamic = Dynamic::from(&json);

   //let mut vec = Vec::with_capacity(10000);
    c.bench_function("Query Language", |b| b.iter(|| black_box(script.eval_with_context(black_box(&mut Context::from([
        ("ROOT", dynamic.clone()),
        ("CURRENT", Dynamic::from(TestObj{
            field1: "TODWA".into(),
            field2: TestObj2{
                field3: 12,
                field4: true,
            }
        }))
    ]))))));
}




criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);