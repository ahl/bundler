use bundler::{Bundle, FileMapLoader, Resolved};

fn main() {
    println!("going");

    // let mut bundle = Bundle::default();
    let mut bundle = Bundle::new(
        FileMapLoader::default()
            .add(
                "https://json-schema.org/draft/2020-12/meta/core"
                    .try_into()
                    .unwrap(),
                "json-2020-12/core".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/applicator"
                    .try_into()
                    .unwrap(),
                "json-2020-12/applicator".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/unevaluated"
                    .try_into()
                    .unwrap(),
                "json-2020-12/unevaluated".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/validation"
                    .try_into()
                    .unwrap(),
                "json-2020-12/validation".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/meta-data"
                    .try_into()
                    .unwrap(),
                "json-2020-12/meta-data".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/format-annotation"
                    .try_into()
                    .unwrap(),
                "json-2020-12/format-annotation".into(),
            )
            .add(
                "https://json-schema.org/draft/2020-12/meta/content"
                    .try_into()
                    .unwrap(),
                "json-2020-12/content".into(),
            ),
    );

    let start_content = include_str!("../../json-2020-12/schema");

    let context = bundle
        .add_content(start_content)
        .expect("must be something wrong with the content?");

    let mut xxx = vec![context];

    // TODO this is just for fun; not quite what we'll need.
    while let Some(context) = xxx.pop() {
        let Resolved {
            context,
            value,
            schema,
        } = bundle.resolve(&context, "#").expect("can't fail");

        bundler::to_generic(&bundle, context, value, schema);
    }

    panic!("got here?");
}
