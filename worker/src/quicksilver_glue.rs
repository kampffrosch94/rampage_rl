use froql::entity_store::Entity;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct EntityWrapper(pub Entity);

impl ::quicksilver::Quicksilver for EntityWrapper {
    const MIRROR: ::quicksilver::Type = ::quicksilver::Type::Struct(&::quicksilver::Struct {
        name: "EntityWrapper",
        size: ::std::mem::size_of::<Self>(),
        align: align_of::<Self>(),
        fields: &[
            ::quicksilver::Field {
                name: "id",
                ty: u32::MIRROR,
                offset: ::std::mem::offset_of!(Self, 0.id),
            },
            ::quicksilver::Field {
                name: "gen",
                ty: u32::MIRROR,
                offset: ::std::mem::offset_of!(Self, 0.generation),
            },
        ],
    });
}

impl From<Entity> for EntityWrapper {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}

const _: () = const {
    // check that there are only id and gen fields in entity
    assert!(size_of::<EntityWrapper>() == 2 * size_of::<u32>());
};

#[cfg(test)]
mod test {
    use super::*;
    use froql::world::World;
    use quicksilver::{json::from_json, reflections_ref::reflect_ref};

    #[test]
    fn test_roundtrip() {
        let mut world = World::new();
        let e = EntityWrapper(world.create_entity());
        let s = reflect_ref(&e).to_json();
        println!("{s}");
        let e2: EntityWrapper = from_json(&s);
        assert_eq!(e.0, e2.0)
    }

    #[test]
    fn test_size_assumption() {}
}
