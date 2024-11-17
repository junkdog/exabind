use std::ops::Deref;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::{CellFilter, CellIterator, Duration, Effect, EffectTimer, RefCount, Shader};

pub type InstanceId = u32;

#[derive(Clone)]
pub struct Unique {
    id_context: RefCount<UniqueContext>,
    instance_id: InstanceId,
    fx: Effect,
}

#[derive(Clone, Debug)]
pub(super) struct UniqueContext {
    pub key: String,
    pub instance_id: InstanceId,
}

impl UniqueContext {
    pub fn new(key: impl Into<String>, instance_id: InstanceId) -> Self {
        Self {
            key: key.into(),
            instance_id,
        }
    }
}

impl Unique {
    pub fn new(id_context: RefCount<UniqueContext>, fx: Effect) -> Self {
        let instance_id = id_context.borrow().deref().instance_id;
        Self {
            id_context,
            instance_id,
            fx,
        }
    }
}

impl Shader for Unique {
    fn name(&self) -> &'static str {
        "unique"
    }

    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) -> Option<Duration> {
        self.fx.process(duration, buf, area)
    }

    fn execute(&mut self, _alpha: f32, _area: Rect, _cell_iter: CellIterator) {}

    fn done(&self) -> bool {
        let binding = self.id_context.borrow();
        let iid = binding.deref().instance_id;
        self.instance_id != iid || self.fx.done()
    }

    fn clone_box(&self) -> Box<dyn Shader> {
        Box::new(self.clone())
    }

    fn area(&self) -> Option<Rect> {
        self.fx.area()
    }

    fn set_area(&mut self, area: Rect) {
        self.fx.set_area(area);
    }

    fn set_cell_selection(&mut self, filter: CellFilter) {
        self.fx.set_cell_selection(filter);
    }

    fn reverse(&mut self) {
        self.fx.reverse();
    }

    fn timer_mut(&mut self) -> Option<&mut EffectTimer> {
        None
    }

    fn timer(&self) -> Option<EffectTimer> {
        self.fx.timer()
    }

    fn cell_selection(&self) -> Option<CellFilter> {
        self.fx.cell_selection()
    }

    fn reset(&mut self) {
        self.fx.reset();
    }
}