use tracing::info;
use turbostate::Engine;
use turbostate::Flow;

#[derive(Debug)]
pub struct In;

#[derive(Debug, Default)]
pub enum State {
	#[default]
	Idle,
	Jump,
	Entry,
	Finalizing,
	WaitRelease,
}

#[derive(Debug, Default)]
pub struct Shared {
	pub hold: bool,
	pub rush: bool,
}

type InFlow = Flow<State, !>;

impl Engine for In {
	type Error = !;
	type Event = (u32, bool);
	type Shared = Shared;
	type State = State;

	fn flow(state: &Self::State, event: Self::Event, shared: &mut Self::Shared) -> InFlow {
		use Flow::*;
		use State::*;

		if let (2, high) = event {
			shared.hold = high;
		}

		if let (1, high) = event {
			shared.rush = high;
		}

		let open = || -> InFlow {
			info!("Сессия открыта");
			info!("Шлагбаум закрыт");
			info!("------------------");
			if shared.rush {
				Transition(Jump)
			} else {
				Transition(Idle)
			}
		};

		match (state, event) {
			(Idle, (1, true)) | (Jump, _) => {
				info!("Проезд инициирован");
				info!("Шлагбаум открыт");
				Transition(Entry)
			}
			(Entry, (3, true)) => {
				info!("Проезд завершается");
				Transition(Finalizing)
			}
			(Finalizing, (3, false)) => {
				if shared.hold {
					Transition(WaitRelease)
				} else {
					open()
				}
			}
			(WaitRelease, (2, false)) => open(),
			(_, (line, _)) => {
				info!("Сигнал на линии [{line}] проигнорирован. Состояние не изменено");
				Pass
			}
		}
	}
}
