use crate::LaurentCtl;
use crate::Recognizer;
use crate::Subscriber;
use std::rc::Rc;
use tracing::info;

/// Состояние цикла
#[derive(Debug, Default)]
pub enum State {
	#[default]
	Idle,
	Entry,
	Finalizing {
		hold: bool,
	},
	WaitRelease,
}

/// Событийный цикл (цикл сессий) на въёзд
#[derive(Debug, Default)]
pub struct InLoop {
	state: State,
	recognizer: Rc<Recognizer>,
	lanes: [bool; 7],
}

impl InLoop {
	pub fn new(recognizer: Rc<Recognizer>) -> Self {
		Self {
			recognizer,
			..Default::default()
		}
	}

	pub fn is_idle(&self) -> bool {
		matches!(self.state, State::Idle)
	}
}

/// Подписка на изменение сигнала
impl Subscriber for InLoop {
	fn ein(&mut self, ctl: &mut LaurentCtl, line: u32, high: bool) {
		use State::*;

		// Сохранение состояния входов
		self.lanes[line as usize] = high;

		match (&self.state, line, high) {
			(Idle, 1, true) => {
				self.state = Entry;
				info!("Проезд инициирован");
				let vrms = self.recognizer.flush();
				info!("Распознанные номера: {vrms:?}");
				// Тут типа создаётся сессия в бд
				info!("Сессия создана");

				// Подаётся сигнал на открытие
				for _ in 0..3 {
					ctl.relay(1, true);
					// thread::sleep(Duration::from_millis(100));
					ctl.relay(1, false);
					// thread::sleep(Duration::from_millis(500));
				}

				info!("Шлагбаум открыт");
			}
			(Entry, 3, true) => {
				self.state = Finalizing {
					// Занят ли на момент выезда второй радар
					hold: self.lanes[2],
				};
				info!("Въезд завершается");
			}
			(Finalizing { hold: true }, 2, false) => {
				info!("Второй радар освобождён");
				self.state = Finalizing { hold: false };
			}
			// Открыть сессию если 3 радар был освобождён и в этот момент 2 радар тоже свободен
			(Finalizing { hold: false }, 3, false) => {
				info!("Проезд завершён. Сессия открыта");
				self.state = Default::default();
				self.lanes = Default::default();

				// Если второй радар ещё не освободился, переключаем на ожидание его
				// освобождения Иначе закрываем шлаг
				if self.lanes[2] {
					info!("Ожидание освобождения второго радара");
					self.state = WaitRelease;
				} else {
					// Подаётся сигнал на закрытие
					for _ in 0..3 {
						ctl.relay(2, true);
						// thread::sleep(Duration::from_millis(100));
						ctl.relay(2, false);
						// thread::sleep(Duration::from_millis(500));
					}

					self.state = Idle;
				}
			}
			(WaitRelease, 2, false) => {
				// Подаётся сигнал на закрытие
				for _ in 0..3 {
					ctl.relay(2, true);
					// thread::sleep(Duration::from_millis(100));
					ctl.relay(2, false);
					// thread::sleep(Duration::from_millis(500));
				}

				info!("Шлагбаум закрыт");

				self.state = Idle;
			}
			(_, line, _) => info!("Сигнал на линии [{line}] проигнорирован. Состояние не изменено"),
		};
	}
}
