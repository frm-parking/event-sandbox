use std::fmt::Debug;
use tracing::debug;

#[derive(Debug, Default)]
pub struct LaurentCtl;

impl LaurentCtl {
	/// Переключает состояние реле
	pub fn relay(&mut self, r: u32, high: bool) {
		if high {
			debug!("Реле [{r}] переведено в открытое состояние");
		} else {
			debug!("Реле [{r}] переведено в закрытое состояние");
		}
	}
}

/// Sandbox реализация интерфейса Laurent
/// ***
/// В реальной реализации:
/// * Есть слой Channel над TCP для обработки сообщений
/// * Сообщения представлены в формате строки
/// * Важен порядок обработки сообщений
#[derive(Debug)]
pub struct Laurent {
	/// Подписчик на события
	sub: Box<dyn Subscriber + 'static>,
	ctl: LaurentCtl,
}

impl Laurent {
	pub fn new(sub: impl Subscriber + 'static) -> Self {
		Self {
			sub: Box::new(sub),
			ctl: LaurentCtl,
		}
	}

	/// Имитирует изменение сигнала на входе
	pub fn emit_ein(&mut self, line: u32, high: bool) {
		if high {
			debug!("Высокий сигнал на линии [{line}]");
		} else {
			debug!("Низкий сигнал на линии [{line}]");
		}

		self.sub.ein(&mut self.ctl, line, high);
	}
}

pub trait Subscriber: Debug {
	/// Событие срабатывает при изменении сигнала на входах
	fn ein(&mut self, ctl: &mut LaurentCtl, line: u32, high: bool);
}
