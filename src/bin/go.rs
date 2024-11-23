use bundler::{Bundle, FileMapLoader, Resolved};

fn main() {
    println!("going");

    // This is the mapping of files referenced to the cached copies on disk.
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

    // TODO why are we not adding this with an ID?
    let context = bundle
        .add_content(start_content)
        .expect("must be something wrong with the content?");

    // let mut xxx = vec![context];

    // TODO this is just for fun; not quite what we'll need.
    // while let Some(context) = xxx.pop() {
    //     let Resolved {
    //         context,
    //         value,
    //         schema,
    //     } = bundle
    //         .resolve(&context, "#")
    //         .expect("resolving the root reference cannot fail");

    //     bundler::to_generic(&bundle, context, value, schema);
    // }

    // TODO I think instead I need to do this:
    // 1. pop the schema
    // 2. convert
    // 3. put any contained subschema (e.g. allOf, oneOf, anyOf) onto the stack
    // 4. maybe also put any other references e.g. to properties into some
    //    other work queue that is just the full list of shit to get to rather
    //    than having some required order
    // ...
    // not sure exactly what happens next, but probably canonicalizing the
    // top of the stack, pop, continue
    //
    // The Bundle is clearly an important abstraction -- all the JSON goo.
    // I think another is going to be something that takes all the schemas
    // and turns them into some canonical form.
    // Finally, perhaps some IR for types that independent of JSON Schema?
    //
    // Progenitor needs some way of saying: "here's some text or here's some
    // serde_json::Value and I know the $schema is X so go do your thing"
    let xxx = bundle
        .resolve(&context, "#")
        .expect("resolving the root reference cannot fail");

    let mut proto = vec![xxx];
    let mut schemas = Vec::new();

    loop {
        if let Some(Resolved {
            context,
            value,
            schema,
        }) = proto.pop()
        {
            println!("got {:?} '{}'", context, schema);
            let schema = bundler::to_ir(value, schema);

            let deps = schema.list_subschema_refs();
            for dep in deps {
                let www = bundle.resolve(&context, dep).expect("bad ref?");
                println!("resolved {:?}", www.context);
                proto.push(www);
            }

            schemas.push(schema);
        } else if let Some(schema) = schemas.pop() {
            println!("{:#?}", schema);
            todo!();
        } else {
            break;
        }
    }

    panic!("got here?");
}
