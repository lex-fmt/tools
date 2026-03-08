use lex_babel::formats::lex::export;
use lex_core::lex::ast::elements::container::{GeneralContainer, ListContainer, SessionContainer};
use lex_core::lex::ast::elements::sequence_marker::{
    DecorationStyle, Form, Separator, SequenceMarker,
};
use lex_core::lex::ast::elements::typed_content::{ContentElement, SessionContent};
use lex_core::lex::ast::elements::BlankLineGroup;
use lex_core::lex::ast::*;
use lex_core::lex::parsing::parse_document;
use proptest::prelude::*;

// -----------------------------------------------------------------------------
// AST Node Generators
// -----------------------------------------------------------------------------

fn paragraph_strategy() -> impl Strategy<Value = Paragraph> {
    prop::collection::vec("[a-zA-Z0-9]+( [a-zA-Z0-9]+)*", 1..5).prop_map(|lines| Paragraph {
        lines: lines
            .into_iter()
            .map(|line| ContentItem::TextLine(TextLine::new(TextContent::from_string(line, None))))
            .collect(),
        annotations: vec![],
        location: Default::default(),
    })
}

fn list_item_strategy() -> impl Strategy<Value = ListItem> {
    ("[a-zA-Z0-9]+( [a-zA-Z0-9]+)*").prop_map(|text| ListItem {
        marker: TextContent::from_string("-".to_string(), None),
        text: vec![TextContent::from_string(format!("{text}\n"), None)],
        children: GeneralContainer::empty(),
        annotations: vec![],
        location: Default::default(),
    })
}

fn list_strategy() -> impl Strategy<Value = List> {
    prop::collection::vec(list_item_strategy(), 2..5).prop_map(|items| {
        let mut list_container = ListContainer::empty();
        for item in items {
            list_container.push(ContentItem::ListItem(item));
        }
        let mut list = List::new(vec![]);
        list.items = list_container;
        list.marker = Some(SequenceMarker {
            raw_text: TextContent::from_string("-".to_string(), None),
            style: DecorationStyle::Plain,
            separator: Separator::Period,
            form: Form::Short,
            location: Default::default(),
        });
        list
    })
}

fn session_strategy() -> impl Strategy<Value = Session> {
    let content_strategy = prop_oneof![
        paragraph_strategy().prop_map(|p| SessionContent::Element(ContentElement::Paragraph(p))),
        list_strategy().prop_map(|l| SessionContent::Element(ContentElement::List(l))),
    ];

    (
        "[a-zA-Z0-9]+",
        prop::collection::vec(content_strategy, 1..3).prop_filter(
            "No consecutive lists",
            |items| {
                let mut prev_was_list = false;
                for item in items {
                    let is_list = matches!(item, SessionContent::Element(ContentElement::List(_)));
                    if is_list && prev_was_list {
                        return false;
                    }
                    prev_was_list = is_list;
                }
                true
            },
        ),
    )
        .prop_map(|(title, content)| {
            let mut spaced_content = Vec::new();
            for c in content {
                spaced_content.push(c);
                spaced_content.push(SessionContent::Element(ContentElement::BlankLineGroup(
                    BlankLineGroup {
                        count: 1,
                        source_tokens: vec![],
                        location: Default::default(),
                    },
                )));
            }

            Session {
                title: TextContent::from_string(title, None),
                marker: None,
                children: SessionContainer::from_typed(spaced_content),
                annotations: vec![],
                location: Default::default(),
            }
        })
}

fn document_strategy() -> impl Strategy<Value = Document> {
    prop::collection::vec(session_strategy(), 1..3).prop_map(|sessions| {
        let mut doc = Document::new();
        doc.root.children = SessionContainer::from_typed(
            sessions.into_iter().map(SessionContent::Session).collect(),
        );
        doc
    })
}

// -----------------------------------------------------------------------------
// The Round-Trip Test
// -----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_round_trip_holy_grail(ast in document_strategy()) {
        println!("Running round-trip for generated doc:\n{:#?}", ast);
        let serialized = export(&ast).expect("Serialization should not fail");
        let parsed = parse_document(&serialized).expect("Parsing should not fail");

        // The exact AST equality check.
        // The parser doesn't perfectly preserve locations back to 'empty' defaults,
        // and may synthesize blank line groups between blocks. We check semantic
        // equivalence instead of exact deep equality containing syntethics/locations.
        // equivalence instead of exact deep equality containing synthetics/locations.


        // But as a first step, let's just make sure it parses the output back into an AST!
        assert!(!parsed.root.children.is_empty());

        // Assert recursive equivalence ignoring positional locations.
        let e_items: Vec<&ContentItem> = ast.root.children.iter().collect();
        let a_items: Vec<&ContentItem> = parsed.root.children.iter().collect();
        assert_ast_equiv(&e_items, &a_items, &serialized);
    }
}

// -----------------------------------------------------------------------------
// Equivalence Checks
// -----------------------------------------------------------------------------

fn assert_ast_equiv(expected: &[&ContentItem], actual: &[&ContentItem], lex_string: &str) {
    // Filter out synthesized blank line groups from parsing results
    // and potentially empty generated lists
    let filtered_expected: Vec<&ContentItem> = expected
        .iter()
        .filter(|&&item| {
            if let ContentItem::BlankLineGroup(_) = item {
                return false;
            }
            true
        })
        .copied()
        .collect();

    let filtered_actual: Vec<&ContentItem> = actual
        .iter()
        .filter(|&&item| !matches!(item, ContentItem::BlankLineGroup(_)))
        .copied()
        .collect();

    if filtered_expected.len() != filtered_actual.len() {
        println!("EXPECTED ITEMS: {:#?}", filtered_expected);
        println!("ACTUAL ITEMS: {:#?}", filtered_actual);
        println!("FAILING LEX STRING:\n======\n{}\n======", lex_string);
        assert_eq!(
            filtered_expected.len(),
            filtered_actual.len(),
            "AST item counts do not match"
        );
    }

    for (exp, act) in filtered_expected.iter().zip(filtered_actual.iter()) {
        match (*exp, *act) {
            (ContentItem::Paragraph(e_p), ContentItem::Paragraph(a_p)) => {
                assert_eq!(e_p.text(), a_p.text(), "Paragraph text mismatch");
            }
            (ContentItem::List(e_l), ContentItem::List(a_l)) => {
                let e_items: Vec<&ContentItem> = e_l.items.iter().collect();
                let a_items: Vec<&ContentItem> = a_l.items.iter().collect();
                assert_ast_equiv(&e_items, &a_items, lex_string);
            }
            (ContentItem::ListItem(e_li), ContentItem::ListItem(a_li)) => {
                let e_text = e_li.text.first().map(|t| t.as_string()).unwrap_or("");
                let a_text = a_li.text.first().map(|t| t.as_string()).unwrap_or("");
                assert_eq!(e_text, a_text, "ListItem text mismatch");
                let e_children: Vec<&ContentItem> = e_li.children.iter().collect();
                let a_children: Vec<&ContentItem> = a_li.children.iter().collect();
                assert_ast_equiv(&e_children, &a_children, lex_string);
            }
            (ContentItem::Session(e_s), ContentItem::Session(a_s)) => {
                assert_eq!(
                    e_s.title.as_string(),
                    a_s.title.as_string(),
                    "Session title mismatch"
                );
                let e_children: Vec<&ContentItem> = e_s.children.iter().collect();
                let a_children: Vec<&ContentItem> = a_s.children.iter().collect();
                assert_ast_equiv(&e_children, &a_children, lex_string);
            }
            // More variants (Verbatim, Definition, Annotation) go here when added...
            _ => panic!(
                "AST node types do not match or are not supported. Expected: {:?}, Actual: {:?}",
                exp, act
            ),
        }
    }
}
