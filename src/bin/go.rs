use std::collections::BTreeMap;

use bundler::{ir, ir2, xxx_to_ir, xxx_to_ir2, Bundle, FileMapLoader, Resolved};

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
    // let xxx = bundle
    //     .resolve(&context, "#")
    //     .expect("resolving the root reference cannot fail");

    // let mut proto = vec![xxx];
    // let mut schemas = Vec::new();

    // loop {
    //     if let Some(Resolved {
    //         context,
    //         value,
    //         schema,
    //     }) = proto.pop()
    //     {
    //         println!("got {:?} '{}'", context, schema);
    //         let schema = bundler::to_ir(value, schema);

    //         let deps = schema.list_subschema_refs();
    //         for dep in deps {
    //             let www = bundle.resolve(&context, dep).expect("bad ref?");
    //             println!("resolved {:?}", www.context);
    //             proto.push(www);
    //         }

    //         schemas.push(schema);
    //     } else if let Some(schema) = schemas.pop() {
    //         println!("{:#?}", schema);
    //         todo!();
    //     } else {
    //         break;
    //     }
    // }

    // Each "raw" schema may represent **many** nested schemas. If we maintin
    // the nesting, then we will need to recursively descend into nested
    // schemas. Instead, we extract each schema in isolation and retain a
    // reference based on the document and path. We represent these isolated
    // schemas in the IR that we manipulate into the eventual canonical form.
    //
    // We start with the root schema.
    // XXX we have at least two kinds of work queues, I think:
    // - completed work in the form of IR in canonical form
    // - initial IR as derived form the raw schemas (many:1)
    // - some way of following references to get more raw schemas
    //
    // Wrt that last category, I could imagine something like a raw schema
    // work queue that we prioritize highest. We'd add new entries to it when
    // trying to canonicalize an IR and finding a reference (maybe?). We also
    // need some idea of dependencies i.e. because we won't be able to
    // canonicalize an IR until we've done so to its dependencies (i.e.
    // transitively). If there's an IR we can't yet handle we somehow need to
    // defer it.
    //
    // Could we somehow get all the raw IR before trying to canonicalize it?
    // The first pass would be to extract all raw schema and produce IR; we
    // would chase references and iterate. Let's try that pass at least...

    // let xxx = bundle
    //     .resolve(&context, "#")
    //     .expect("resolving the root reference cannot fail");

    // bundler::xxx_to_ir(&xxx);

    // let mut work = vec![(context, "#".to_string())];

    // let mut yyy = BTreeMap::new();

    // while let Some((context, path)) = work.pop() {
    //     println!();
    //     println!("got work");
    //     // Step 1: resolve the entity. This will give us a raw value
    //     let xxx = bundle.resolve(&context, &path).expect("failed to resolve?");

    //     // Step 2: from the raw value, produce a number of IRs, one for each
    //     // of the distinct schemas (i.e. aware of all the nesting). We should
    //     // minimize the amount of effort that goes into generating each. We
    //     // need to save all these IRs somewhere.
    //     let irs = xxx_to_ir(&xxx).unwrap();

    //     println!("irs? {:#?}", irs);

    //     // Step 3: one IR variant is going to be for a bare $ref. For each of
    //     // those dump a new item of work into the queue (current context +
    //     // reference string)

    //     for (rr, ir) in irs {
    //         if let ir::SchemaDetails::DollarRef(ref_target) = &ir.details {
    //             println!("ref: {}", ref_target);
    //             work.push((xxx.context.clone(), ref_target.clone()));
    //         }
    //         yyy.insert(rr, ir);
    //     }
    // }

    // println!("all");
    // for (k, v) in yyy {
    //     println!("{:#?}", k);
    //     println!("{}", serde_json::to_string_pretty(&v).unwrap());
    // }

    // 12/15/2024
    // I think we want to try this again and be a little more methodical now
    // that we have a better idea of what we're doing.
    //
    // For a while I've been back and forth about whether we want to try to
    // process large objects or to strip them apart into minimal, stand-alone
    // schemas. There are some constructions that--for some reason--led me to
    // believe that the segmented approach wasn't feasible, but now I'm fairly
    // certain that both could be made to work. An important observation is
    // that loops may (and do!) exist in these schemas, even the meta schema
    // for 2020-12.
    //
    // Consider the `dependencies` property that contains a anyOf [ #meta,
    // stringArray ]. If putting the meta schema into a canonical state
    // requires us to do so for its properties, and resolving the #meta dynamic
    // dependency requires us to have put **that** in its canonical state, we
    // can never resolved it!
    //
    // However! I think we can--effectively--keep simplifying until no further
    // simplifications are possible, then do some speculative reference
    // resolution. In this case if we were to do our best with the meta schema
    // (i.e. if it had not reached its canonical state), there would still be
    // enough to determine that the anyOf was effectively an xorOf by seeing
    // that stringArray was incompatible with type: [object, boolean].

    // At the end of this loop, we'll have all the unprocessed--i.e.
    // non-canonical--IRs in some structure indexed by SchemaRef (whatever that
    // ends up being). At this point we can start to canonicalize them. We can
    // start wherever and figure out dependencies (I think?).

    let mut references = vec![(context, "#".to_string())];

    let mut raw = BTreeMap::new();

    while let Some((context, reference)) = references.pop() {
        println!();
        println!("got work: {} {}", context.id, reference);

        let resolved = bundle
            .resolve(&context, &reference)
            .expect("failed to resolve reference");
        println!("resolved: {:#?}", resolved);

        let xxx = ir2::SchemaRef::Id(resolved.context.id.clone());
        if let Some(yyy) = raw.get(&xxx) {
            println!("context: {:#?}", context);
            println!("reference: {}", reference);
            println!("already have {xxx}");
            println!("{:#?}", yyy);
            continue;
        }

        let irs = xxx_to_ir2(&resolved).unwrap();

        for (sr, ir) in irs {
            println!("sr = {sr}");

            if let ir2::Schema::DollarRef(ref_target) = &ir {
                println!("$ref => {}", ref_target);
                references.push((resolved.context.clone(), ref_target.clone()));
            }

            let xxx = raw.insert(sr, ir);
            assert!(xxx.is_none());
        }
    }

    for (k, v) in &raw {
        println!("sr {}", k);
    }

    for (k, v) in &raw {
        println!("{}: {}", k, serde_json::to_string_pretty(v).unwrap());
    }

    let mut done = BTreeMap::new();
    for (k, v) in raw {
        match v.simplify() {
            ir2::State::Canonical(schema) => {
                done.insert(k, schema);
            }
            ir2::State::Simplified(schema) => todo!(),
            ir2::State::Stuck(schema) => todo!(),
            ir2::State::Todo => (),
        }
    }
    println!();
    println!("DONE");
    println!();
    for (k, v) in &done {
        println!("{}: {}", k, serde_json::to_string_pretty(v).unwrap());
    }

    panic!("got here?");
}

// 11/26/2024 I'm struggling with a dilemma:
//
// I can model the IR in two distinct ways:
//
// Self-Contained
// --------------
//
// Each Schema would contain SchemaRef notes that, effectively, index into some
// lookup table of resolved IR. To process an IR, I'd make sure its
// dependencies were already resolved (and if not, defer the current IR and
// schedule those). What I'm not sure of is ... then what? Would I try to
// stitch the schemas back together into a deeper form? Does that even make
// sense?
//
// Maybe it makes sense to think about the canonical form I expect to get
// after processing the IR?
//
// Deep Trees
// ----------
//
// The other approach is to more faithfully represent the input tree as a deep
// structure. To turn this into an IR, we'd probably need to chew on it until
// we hit something that wasn't resolved, then back out, scheduled the
// dependent work, and pick it up again. It seems sort of inefficient... but
// maybe it's not so bad.
