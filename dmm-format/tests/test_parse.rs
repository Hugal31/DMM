use dmm::Datum;
use std::collections::HashMap;

#[test]
fn test_parse_str() {
    let source = r#"// Comment
"aaa" = (
/turf/open/space/basic,
/area/space),
"aab" = (
/obj/machinery/firealarm{
        dir = 8;
        name = "thing"
        }
)

(1,1,1) = {"
aaa
aab
"}
"#;
    assert_eq!(
        dmm_format::from_str(source).expect("Should have parsed"),
        dmm::DMM::new(
            {
                let mut m = HashMap::new();
                m.insert(
                    0.into(),
                    vec![
                        Datum::new("/turf/open/space/basic"),
                        Datum::new("/area/space"),
                    ],
                );
                m.insert(
                    1.into(),
                    vec![Datum::with_var_edits(
                        "/obj/machinery/firealarm",
                        vec![
                            ("dir".to_string(), dmm::Literal::Number(8)),
                            ("name".to_string(), dmm::Literal::Str("thing".to_string())),
                        ]
                        .into_iter()
                        .collect(),
                    )],
                );
                m
            },
            {
                let mut m = HashMap::new();
                m.insert((1, 1, 1), vec![0.into(), 1.into()]);
                m
            }
        )
    );
}

#[test]
fn test_parse_reader() {
    let file = std::fs::File::open("tests/multiz.dmm").expect("file should be here");
    assert!(dbg!(dmm_format::from_reader(file)).is_ok());
}
