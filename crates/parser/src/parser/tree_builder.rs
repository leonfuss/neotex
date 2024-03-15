use rowan::GreenNodeBuilder;

use super::parser::ParserEvent;

fn build_tree(events: Vec<ParserEvent>) -> rowan::GreenNode {
    let mut events = events;
    let builder = GreenNodeBuilder::new();

    // TODO: follow open_before links before evaluating close events

    assert!(matches!(events.pop(), Some(ParserEvent::Close { .. })));

    for event in events {}

    todo!()
}
