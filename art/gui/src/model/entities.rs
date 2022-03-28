#![allow(unused)]

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use ves_art_core::sprite::Animation;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Entities(
    LinkedHashMap<Cow<'static, str>, Entity>,
);

impl Entities {
    pub fn push(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        entity: Entity,
    ) -> Result<(), String> {
        let name = name.into();
        if self.0.contains_key(&name) {
            return Err(format!("Attempt at adding a duplicate entity: {}", &name));
        }

        self.0.insert(name, entity);

        Ok(())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Entity)> {
        self.0.iter()
    }

    pub fn get(&self, key: &Cow<'static, str>) -> Option<&Entity> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &Cow<'static, str>) -> Option<&mut Entity> {
        self.0.get_mut(key)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Entity {
    animations: Animations,
}

impl Entity {
    /// Retrieves the [`Animations`].
    #[allow(unused)]
    pub fn animations(&self) -> &Animations {
        &self.animations
    }

    /// Retrieves the [`Animations`] mutably.
    pub fn animations_mut(&mut self) -> &mut Animations {
        &mut self.animations
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Animations(
    LinkedHashMap<Cow<'static, str>, Animation>,
);

impl Animations {
    pub fn push(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        animation: Animation,
    ) -> Result<(), String> {
        let name = name.into();
        if self.0.contains_key(&name) {
            return Err(format!(
                "Attempt at adding a duplicate animation: {}",
                &name
            ));
        }

        self.0.insert(name, animation);

        Ok(())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Animation)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ves_art_core::sprite::{AnimationFrame, CelRef};

    const ENTITIES_RON: &'static str = "resources/test/components/entities/entities.ron";

    fn assert_serializes_to(entities: &Entities, path: &'static str) {
        let mut buffer = Vec::new();
        ron::ser::to_writer_pretty(&mut buffer, &entities, ron::ser::PrettyConfig::default())
            .unwrap();
        let actual = String::from_utf8(buffer).unwrap();
        let expected = std::fs::read_to_string(path).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_serialize() {
        use ves_cache::FromIndex as _;
        let mut yoshi = Entity::default();
        {
            let anims = yoshi.animations_mut();
            {
                let mut animation = Animation::default();
                let frames = animation.as_mut();
                frames.push(AnimationFrame::new(CelRef::from_index(12)));
                frames.push(AnimationFrame::new(CelRef::from_index(13)));
                frames.push(AnimationFrame::new(CelRef::from_index(14)));
                frames.push(AnimationFrame::new(CelRef::from_index(15)));
                anims.push("walk", animation).unwrap();
            }
            {
                let mut animation = Animation::default();
                let frames = animation.as_mut();
                frames.push(AnimationFrame::new(CelRef::from_index(77)));
                frames.push(AnimationFrame::new(CelRef::from_index(88)));
                frames.push(AnimationFrame::new(CelRef::from_index(99)));
                anims.push("run", animation).unwrap();
            }
        }

        let mut shy_guy = Entity::default();
        {
            let anims = shy_guy.animations_mut();
            {
                let mut animation = Animation::default();
                let frames = animation.as_mut();
                frames.push(AnimationFrame::new(CelRef::from_index(100)));
                frames.push(AnimationFrame::new(CelRef::from_index(200)));
                frames.push(AnimationFrame::new(CelRef::from_index(300)));
                frames.push(AnimationFrame::new(CelRef::from_index(400)));
                anims.push("walk", animation).unwrap();
            }
            {
                let mut animation = Animation::default();
                let frames = animation.as_mut();
                frames.push(AnimationFrame::new(CelRef::from_index(901)));
                frames.push(AnimationFrame::new(CelRef::from_index(902)));
                frames.push(AnimationFrame::new(CelRef::from_index(903)));
                anims.push("dance", animation).unwrap();
            }
        }

        let mut entities = Entities::default();
        entities.push("yoshi", yoshi).unwrap();
        entities.push("shy_guy", shy_guy).unwrap();

        assert_serializes_to(&entities, ENTITIES_RON);
    }

    #[test]
    fn test_deserialize() {
        let input = std::fs::File::open(ENTITIES_RON).unwrap();
        let entities: Entities = ron::de::from_reader(input).unwrap();

        assert_serializes_to(&entities, ENTITIES_RON);
    }
}
