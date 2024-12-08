use std::fmt::Debug;
use std::ops::Deref;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::{CellFilter, Duration, Effect, EffectTimer, RefCount, Shader};

pub type InstanceId = u32;

#[derive(Clone, Debug)]
pub struct Unique<K: Clone> {
    id_context: RefCount<UniqueContext<K>>,
    instance_id: InstanceId,
    fx: Effect,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(super) struct UniqueContext<K: Clone> {
    pub key: K,
    pub instance_id: InstanceId,
}

impl<K: Clone> UniqueContext<K> {
    pub fn new(key: impl Into<K>, instance_id: InstanceId) -> Self {
        Self {
            key: key.into(),
            instance_id,
        }
    }
}

impl<K: Clone> Unique<K> {
    pub fn new(id_context: RefCount<UniqueContext<K>>, fx: Effect) -> Self {
        let instance_id = id_context.borrow().deref().instance_id;
        Self {
            id_context,
            instance_id,
            fx,
        }
    }
}

impl<K: Clone + Debug + 'static> Shader for Unique<K> {
    fn name(&self) -> &'static str {
        "unique"
    }

    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) -> Option<Duration> {
        self.fx.process(duration, buf, area)
    }

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