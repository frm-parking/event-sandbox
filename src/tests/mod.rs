use crate::laurent::Laurent;
use crate::sloop::InLoop;
use crate::Recognizer;
use std::rc::Rc;

#[test]
fn in_loop_perfect_conditions() {
	tracing_subscriber::fmt::init();
	let recognizer = Rc::new(Recognizer::default());
	recognizer.add_vrm("X777XX77");
	recognizer.add_vrm("A121AA12");
	let lp = InLoop::new(recognizer.clone());
	let mut laurent = Laurent::new(lp);
	laurent.emit_ein(1, true);
	laurent.emit_ein(2, true);
	laurent.emit_ein(1, false);
	laurent.emit_ein(3, true);
	laurent.emit_ein(2, false);
	laurent.emit_ein(3, false);
}
