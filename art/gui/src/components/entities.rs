use linked_hash_map::LinkedHashMap;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;
use ves_art_core::sprite::Animation;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Entities(
    #[serde(
        serialize_with = "serialize_linked_hash_map",
        deserialize_with = "deserialize_linked_hash_map"
    )]
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
    #[serde(
        serialize_with = "serialize_linked_hash_map",
        deserialize_with = "deserialize_linked_hash_map"
    )]
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
}

fn serialize_linked_hash_map<A, B, S>(
    map: &LinkedHashMap<A, B>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    A: Serialize + Eq + Hash,
    B: Serialize,
{
    let mut ser = serializer.serialize_map(Some(map.len()))?;
    for (key, value) in map {
        ser.serialize_entry(key, value)?;
    }
    ser.end()
}

struct LinkedHashMapVisitor<K, V> {
    marker: PhantomData<fn() -> LinkedHashMap<K, V>>,
}

impl<K, V> LinkedHashMapVisitor<K, V> {
    fn new() -> Self {
        Self {
            marker: PhantomData::default(),
        }
    }
}

impl<'de, K, V> Visitor<'de> for LinkedHashMapVisitor<K, V>
where
    K: Deserialize<'de> + Hash + Eq,
    V: Deserialize<'de>,
{
    type Value = LinkedHashMap<K, V>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut map = LinkedHashMap::with_capacity(map_access.size_hint().unwrap_or(0));

        while let Some((key, value)) = map_access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

fn deserialize_linked_hash_map<'de, D, K, V>(
    deserializer: D,
) -> Result<LinkedHashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Hash + Eq,
    V: Deserialize<'de>,
{
    deserializer.deserialize_map(LinkedHashMapVisitor::new())
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
