use tracing::info;
use turbostate::Engine;
use turbostate::Flow;

/// Сценарий на вхезд
#[derive(Debug)]
pub struct In;

/// Состояние
#[derive(Debug, Default)]
pub enum State {
	/// Въезд ожидается
	#[default]
	Idle,
	/// Проезд инициирован
	Entry,
	/// Завершение проезда
	Finalizing,
	/// Если второй рада был занят на момент выезда
	/// нужно подождать его освобождения перед закрытием шлага
	WaitRelease,
}

/// Промежуточные данные, которые нужно сохранять при переходах
/// hold - состояние второго радара (удержание)
/// rust - состояние первого радара (час пик)
#[derive(Debug, Default)]
pub struct Shared {
	pub hold: bool,
	pub rush: bool,
}

#[derive(Debug)]
pub enum Event {
	Ein(u32, bool),
	Rush,
}
/// В текущем сценарии ошибки не возникает
type Error = !;
type InFlow = Flow<State, Error, Event>;

impl Engine for In {
	type Error = Error;
	type Event = Event;
	type Shared = Shared;
	type State = State;

	fn flow(state: &Self::State, event: Self::Event, shared: &mut Self::Shared) -> InFlow {
		use Event::*;
		use Flow::*;
		use State::*;

		// Обновление информации о втором радаре (удержание)
		if let Ein(2, high) = event {
			shared.hold = high;
		}

		// Обновление информации о первом радаре (час пик)
		if let Ein(1, high) = event {
			shared.rush = high;
		}

		// Логика открытия используется в двух ветках
		// Поэтому вынесена в эту функцию
		let open = || -> InFlow {
			info!("Сессия открыта");
			info!("Шлагбаум закрыт");
			info!("------------------");
			if shared.rush {
				// Немедленное переключение на указанную ветку
				Branch(Idle, Rush)
			} else {
				Transition(Idle)
			}
		};

		match (state, event) {
			(Idle, Ein(1, true) | Rush) => {
				info!("Проезд инициирован");
				info!("Шлагбаум открыт");
				Transition(Entry)
			}
			(Entry, Ein(3, true)) => {
				info!("Проезд завершается");
				Transition(Finalizing)
			}
			(Finalizing, Ein(3, false)) => {
				if shared.hold {
					Transition(WaitRelease)
				} else {
					open()
				}
			}
			(WaitRelease, Ein(2, false)) => open(),
			(_, Ein(line, _)) => {
				info!("Сигнал на линии [{line}] проигнорирован. Состояние не изменено");
				// Изменение состояния не требуется
				Pass
			}
			_ => Pass,
		}
	}
}
