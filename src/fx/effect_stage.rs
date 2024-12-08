use crate::fx::unique::{Unique, UniqueContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::BTreeMap;
use std::fmt::Debug;
use tachyonfx::{ref_count, Duration, Effect, IntoEffect, RefCount, Shader, SimpleRng};

/// A stage that manages a collection of terminal UI effects, including uniquely
/// identified effects that can be replaced/cancelled by new effects with the same ID.
///
/// The `EffectStage` provides lifecycle management for both regular effects and unique effects.
/// Regular effects run until completion, while unique effects can be cancelled when a new effect
/// with the same identifier is added.
#[derive(Default)]
pub struct EffectStage<K: Clone + Ord + 'static> {
    effects: Vec<Effect>,
    uniques: BTreeMap<K, RefCount<UniqueContext<K>>>,
    rng: SimpleRng,
}

impl<K: Clone + Debug + Ord> EffectStage<K> {
    /// Creates a unique effect that will cancel any existing effect with the same key.
    /// The effect must be added to the stage using [`add_effect`] to be processed.
    ///
    /// When a new unique effect is created with a key that matches an existing effect,
    /// the existing effect will be marked as complete on the next processing cycle.
    ///
    /// # Arguments
    /// * `key` - A unique identifier for the effect. If an effect with this key already exists,
    ///           the existing effect will be cancelled.
    /// * `fx` - The effect to be wrapped with unique identification.
    ///
    /// # Returns
    /// A new effect that includes unique identification logic. The effect must still be added
    /// to the stage to be processed.
    pub fn unique(&mut self, key: impl Into<K>, fx: Effect) -> Effect {
        let key = key.into();
        let ctx = self.uniques.entry(key.clone())
            .and_modify(|ctx| ctx.borrow_mut().instance_id = self.rng.gen())
            .or_insert_with(|| ref_count(UniqueContext::new(key.clone(), self.rng.gen())))
            .clone();

        Unique::new(ctx, fx).into_effect()
    }

    /// Adds an effect to be processed by the stage.
    ///
    /// The effect will be processed each frame until it is complete.
    ///
    /// # Arguments
    /// * `effect` - The effect to add to the stage
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Creates and adds a unique effect to the stage in a single operation.
    ///
    /// This is a convenience method that combines [`unique`] and [`add_effect`].
    /// Any existing effect with the same key will be cancelled.
    ///
    /// # Arguments
    /// * `key` - A unique identifier for the effect. If an effect with this key already exists,
    ///           the existing effect will be cancelled.
    /// * `fx` - The effect to be wrapped with unique identification and added to the stage.
    pub fn add_unique_effect(&mut self, key: impl Into<K>, fx: Effect) {
        let fx = self.unique(key, fx);
        self.add_effect(fx);
    }

    /// Processes all active effects for the given duration.
    ///
    /// This method should be called each frame in your render loop. It will:
    /// 1. Process each effect for the specified duration
    /// 2. Remove completed effects
    /// 3. Clean up any orphaned unique effect contexts
    ///
    /// # Arguments
    /// * `duration` - The time elapsed since the last frame
    /// * `buf` - The buffer to render effects into
    /// * `area` - The area within which effects should be rendered
    pub fn process_effects(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        self.effects.retain_mut(|effect| {
            effect.process(duration, buf, area);
            effect.running()
        });

        // clear orphaned unique effects;
        self.uniques.retain(|_, ctx| RefCount::strong_count(ctx) > 1);
    }
}
