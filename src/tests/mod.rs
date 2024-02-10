use crate::sloop::Event;
use crate::sloop::In;
use turbostate::Machine;

#[test]
fn in_loop_perfect_conditions() {
	tracing_subscriber::fmt::init();

	let machine = Machine::<In>::default_shared(Default::default());

	let seq = &[
		(1, true),
		(2, true),
		(1, false),
		(3, true),
		(2, false),
		(3, false),
	];

	for &(line, high) in seq {
		machine.fire(Event::Ein(line, high)).unwrap();
	}
}
