use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bundler::{
    ir, ir2,
    namespace::{self, Namespace},
    schemalet::{schemalet_print, schemalet_to_type, to_schemalets},
    typespace::{Typespace, TypespaceBuilder},
    xxx_to_ir, xxx_to_ir2, Bundle, FileMapLoader, Resolved,
};
use url::Url;

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

    // ir2(bundle, context);

    schemalet(bundle, context);
}

fn ir2(bundle: Bundle, context: bundler::Context) {
    let root_id = ir2::SchemaRef::Id(format!("{}#", context.location));

    let mut references = vec![(context, "#".to_string())];

    let mut raw = BTreeMap::new();

    while let Some((context, reference)) = references.pop() {
        println!();
        println!("got work: {} {}", context.location, reference);

        let resolved = bundle
            .resolve(&context, &reference)
            .expect("failed to resolve reference");
        println!("resolved: {:#?}", resolved);

        let xxx = ir2::SchemaRef::Id(resolved.context.location.to_string());
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

    /*
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
    */

    // 1/10/2025
    // I guess the goal of this pass should be to simplify everything as much
    // as possible without resolving dynamic references.

    println!("simplifying");
    let mut wip = raw;
    let mut done = BTreeMap::new();
    let mut pass = 0;
    loop {
        if wip.is_empty() {
            break;
        }
        pass += 1;
        println!("new pass: {pass}");
        let mut next = BTreeMap::new();
        let mut simplified = false;
        for (k, v) in wip {
            println!("{}", k);
            match v.simplify(&done) {
                ir2::State::Canonical(schema) => {
                    println!("simplified");
                    simplified = true;
                    done.insert(k, schema);
                }
                ir2::State::Stuck(schema) => {
                    println!("stuck {}", serde_json::to_string_pretty(&schema).unwrap());
                    next.insert(k, schema);
                }
                ir2::State::Todo => (),
                ir2::State::Simplified(schema, vec) => {
                    println!("simplified");
                    next.insert(k, schema);
                    for (k, v) in vec {
                        next.insert(k, v);
                    }
                    simplified = true;
                }
            }
        }

        wip = next;

        for (k, v) in &done {
            println!("ss {}", k);
            if !v.is_canonical(&done) {
                println!(
                    "not canonical {}",
                    serde_json::to_string_pretty(&k).unwrap()
                );
            }
        }

        // Somehow we got stuck and couldn't simplify any further.
        if !simplified {
            // panic!();
            break;
        }
    }

    println!("\npieces we wanted\n");

    let mut wip = vec![root_id];
    let mut already = BTreeSet::new();
    while let Some(schema_ref) = wip.pop() {
        println!("sr = {schema_ref}");
        if !already.insert(schema_ref.clone()) {
            continue;
        }
        println!("{}", schema_ref);
        let schema = done.get(&schema_ref).unwrap();

        println!("{}", serde_json::to_string_pretty(schema).unwrap());

        let children = schema.children();
        println!("children: {:#?}", children);
        wip.extend(children);
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
//
// 5/9/2025
// I made this decision effectively a while ago, but we've picked
// self-contained. In the new iteration I'm trying, I'm calling these
// "Schemalets".
//
// 5/9/2025
// What are the options for dealing with dynamic references? When we walk the
// graph the first time, the context comes along for the ride which might be
// useful for identifying the appropriate dynamic reference target.
// Alternatively, we could record dynamic inputs and outputs, stitching it all
// together later. While I was hoping to have discrete passes with distinct
// functionality, perhaps dealing with dyn refs on the first pass is simplest.

// 5/9/2025
// Starting yet another attempt that I'm hoping can be cleaner and more
// complete. We're going to try to blaze it all the way through to a canonical
// representation and take the shortest route with dynamic references.

// 6/14/2025
// Working on the "schemalet" model and it's ok. I think I've lost track of
// preserving metadata and of simplifying the canonical name of a schema e.g.
// if merging schemas results in exactly one or the other. Some of this might
// be fixable incrementally, so I really need to push through.
// The "CanonicalSchemalet" structure seems really right. I expect that's going
// to be the output and I should spend time making that thing pretty precisely
// what I need.

// 6/21/2025
// Some new thoughts after not working on this for a week. Increasingly, I've
// come to realize that what we have is a graph (potentially, with cycles), and
// I need to think about this as local graph optimization and modification. So
// for each simplification step, I should have access to the full graph. Each
// node needs more than the one distinction I have today (canonical or not).
// For example, when simplifying an object, I want to say "might any of your
// required properties turn out to be non-satisfiable (never)? If not, then I
// don't care that some properties are non-canonical, I can declare you
// canonical." Other criteria might include "do I know your type?". That said,
// I want to burn through this current draft and see if I can actually start
// generating some code, and figure out the right layering of the various
// pieces.
// Also [6/23/2025] we can keep track of back edges to know what updating a
// node might unblock in terms of outstanding work.

// 6/21/2025
// It's not pretty, but everything is in a canonical form. The next step is to
// do what I think of as the work of typify: translating schemas to types.
//
// - Raw JSON schemas -> schema graph of canonical schemalets.
// - schema graph -> IR for Rust types
// - IR -> generated code

// 6/22/2025
// What types deserve to have a name regardless of whether it's necessary?
// Currently, we pull out the contents of definitions. In this new version,
// definitions are just... another path. But perhaps that's still a good way to
// determine when type names are meaningful.

// 6/23/2025
// Let's flesh out this idea of the pipeline / layers. In particular, what do
// we start with? I assume a consumer is going to make a Bundle with some
// document or documents. And then it's going to specify some collection of
// types to generate using that bundle as source information (either by saying
// "this path, and that path" or "all the $defs" or "matching this pattern").
// In the case of progenitor, each added type is going to require some response
// so the generated type can later be used. This seems easy enough by using the
// SchemaRef.
//
// Either we'll take that "bundle + type specifiers" and destructively convert
// it to some sort of collection of types or shove them through one at a
// time--I'm not sure it matters. Either in serial items or serial batches,
// we'll convert from Raw -> canonical -> type IR
//
// The real question I'm noodling on is "can I have some object whose purpose
// is to produce type structures for all of its input schemas?". Like, I build
// it with this graph of canonical schemas and it just chews through them.

// 6/24/2025
// Two neat ideas:
// 1. (simple) make a printer and/or transformer that serializes them into a
// single object i.e. inlining references. This would obviously be a problem
// for circular references... so may be I infer as much from the "Reference"
// node. Anyhow: print it out as one big chunk.
// 2. Figure out a more robust definition of the Reference type. And maybe add
// some sort of "Replace Me" node for situations where a computed type ends up
// resolving down to some known type and there's no additional information
// added.

// 6/27/2025
// Making good progress, but also kicking over a bunch of stuff along the way.
// I'm really trying to put my head down to get some code generated and then go
// back and think about all the problems. Some notes:
//
// [ ] I should get really robust about the preservation of metadata and make
//     it clear in the canonical output how that will be represented.
//
// [ ] I need to figure out the actual mechanism for resolving name conflicts
//     and injecting new names. Like: how do we detect a name conflict? How do
//     we report it to the user? What does the user do specifically to fix it?
//
// [ ] I need to preserve extensions and I have no idea how I'm going to do
//     that. Maybe I **don't** need to at all because I can always go back to
//     the original schema to see if they exist? But How does that work when I
//     merge schemas?
//
// [ ] How do I know when a type is going to have its own name or not? This
//     seems most relevant for enum struct-type variants, but it seems like we
//     could also do something for tuple fields (i.e. inline them if there
//     isn't going to be some good name).
//
// I'd just note the things that this does maybe already address or is on the
// way to addressing:
// - I think I can be a lot more robust and efficient wrt merging
// - I am keeping around a bunch of breadcrumbs that should let me make better
//   errors (I just need to figure out the details)
// - Multiple files
// - Multiple JSON Schema specifications
// - Generalized references (i.e. not just to $defs)

fn schemalet(bundle: Bundle, context: bundler::Context) {
    let root_id = bundler::schemalet::SchemaRef::Id(format!("{}#", context.location));

    let mut references = vec![(context, "#".to_string())];

    let mut raw = BTreeMap::new();

    while let Some((context, reference)) = references.pop() {
        println!();
        println!("got work: {} {}", context.location, reference);

        let resolved = bundle
            .resolve(&context, &reference)
            .expect("failed to resolve reference");

        if raw.contains_key(&bundler::schemalet::SchemaRef::Id(
            resolved.context.location.to_string(),
        )) {
            continue;
        }

        let schemalets = to_schemalets(&resolved).unwrap();

        for (sref, schemalet) in schemalets {
            println!("sref = {}", sref);

            let schemalet = match schemalet {
                // I've decided that the final "raw" form should have relative
                // references resolved. This makes some of the logic ... into
                // an opportunity for greater consistency!
                bundler::schemalet::Schemalet {
                    details: bundler::schemalet::SchemaletDetails::RawRef(target),
                    metadata,
                } => {
                    let resolved_target = bundle
                        .resolve(&resolved.context, &target)
                        .expect("failed to resolved reference")
                        .context
                        .location;
                    println!("$ref => {target} {resolved_target}");
                    references.push((resolved.context.clone(), resolved_target.to_string()));
                    bundler::schemalet::Schemalet {
                        details: bundler::schemalet::SchemaletDetails::ResolvedRef(
                            bundler::schemalet::SchemaRef::Id(resolved_target.to_string()),
                        ),
                        metadata,
                    }
                }

                // When we hit a dynamic reference, we resolve it right here and
                // now. This is imperfect in some ways, but suffices for the
                // singular use of $dynamicRef that we know of and/or care about.
                bundler::schemalet::Schemalet {
                    details: bundler::schemalet::SchemaletDetails::RawDynamicRef(target),
                    metadata,
                } => {
                    let resolved = context.dyn_resolve(&target).clone();
                    println!("$dynReference => {target} {resolved}");
                    bundler::schemalet::Schemalet {
                        details: bundler::schemalet::SchemaletDetails::ResolvedDynamicRef(
                            bundler::schemalet::SchemaRef::Id(resolved.to_string()),
                        ),
                        metadata,
                    }
                }

                schemalet => schemalet,
            };

            let old = raw.insert(sref, schemalet);
            assert!(old.is_none());
        }
    }

    for (k, _) in &raw {
        println!("sr {}", k);
    }

    for (k, v) in &raw {
        println!("{}: {}", k, serde_json::to_string_pretty(v).unwrap());
    }

    // 5/25/2025
    // Now we're going to process the schemalets until they are all in
    // canonical form. This may take several passes to converge.

    let mut wip = raw;
    let mut canonical = BTreeMap::new();
    let mut pass = 0;

    loop {
        if wip.is_empty() {
            break;
        }

        pass += 1;
        println!("new pass: {pass}");
        let mut next = BTreeMap::new();
        let mut simplified = false;
        for (k, v) in wip {
            println!("simplifying {k}");
            match v.simplify(&canonical) {
                bundler::schemalet::State::Canonical(schemalet) => {
                    println!("canonical");
                    println!("{}", serde_json::to_string_pretty(&schemalet).unwrap());
                    simplified = true;
                    canonical.insert(k, schemalet);
                }

                bundler::schemalet::State::Stuck(schemalet) => {
                    next.insert(k, schemalet);
                }
                bundler::schemalet::State::Simplified(schemalet, items) => {
                    simplified = true;
                    next.insert(k, schemalet);
                    for (new_k, new_v) in items {
                        next.insert(new_k, new_v);
                    }
                }
            }
        }

        wip = next;

        if !simplified {
            panic!("couldn't simplify more");
        }
    }

    for (schema_ref, schemalet) in &canonical {
        println!("canonical {}", schema_ref);
        println!("{}", serde_json::to_string_pretty(schemalet).unwrap());
    }

    typify(canonical, root_id);
    todo!("<fin>");
}

fn typify(
    mut canonical: BTreeMap<bundler::schemalet::SchemaRef, bundler::schemalet::CanonicalSchemalet>,
    root_id: bundler::schemalet::SchemaRef,
) {
    schemalet_print(&canonical, &root_id);

    let schemalet = canonical.get(&root_id).unwrap();

    println!("{}", serde_json::to_string_pretty(schemalet).unwrap());

    let mut typespace = TypespaceBuilder::<bundler::schemalet::SchemaRef>::default();

    canonical.insert(
        bundler::schemalet::SchemaRef::Internal("string".to_string()),
        bundler::schemalet::CanonicalSchemalet {
            metadata: Default::default(),
            details: bundler::schemalet::CanonicalSchemaletDetails::Value(
                bundler::schemalet::SchemaletValue::String {
                    pattern: None,
                    format: None,
                },
            ),
        },
    );

    // TODO 6/30/2025
    // My thought right now on naming is to do some initial configuration of
    // names here i.e. before doing the conversion. This would include the root
    // schema's name, $defs, and the targets of references.
    // It does seem like naming the root schema might suffice and would be
    // worth trying first to stress test dependent naming. Each child should
    // be able to have a name derived from that of its parent.

    let mut converter = bundler::convert::Converter::new(canonical);

    converter.set_name(root_id.clone(), "SchemaRoot".to_string());

    let mut work = VecDeque::from([root_id.clone()]);

    let mut done = BTreeSet::<bundler::schemalet::SchemaRef>::new();

    while let Some(id) = work.pop_front() {
        if done.contains(&id) {
            continue;
        }
        done.insert(id.clone());

        // TODO 7/2/2025
        // Not sure if this is the right place to look for a name, but maybe
        // it's okay. At this point we know that the path really is about to
        // have a type at this location, and we don't know that any sooner.
        //
        // Note that we need to have something more generic than $defs and I'm
        // not sure we're always going to apply this heuristic.
        //
        // TODO 7/10/2025
        // In sum: there's more thinking to do here.
        if let bundler::schemalet::SchemaRef::Id(path) = &id {
            let url = Url::parse(path).unwrap();

            if let Some(fragment) = url.fragment() {
                if let Some(name) = fragment.strip_prefix("/$defs/") {
                    converter.set_name(id.clone(), name.to_string());
                }
            }
        }

        println!("id = {id}");

        let typ = converter.convert(&id);

        typespace.insert(id.clone(), typ.clone());

        println!("{:#?}", &typ);

        work.extend(typ.children());
    }

    let out = typespace.render();
    println!("file\n{out}");

    let typespace_final = typespace.finalize().unwrap();
    let out = typespace_final.render();
    println!("file\n{out}");

    todo!("done?");
}
