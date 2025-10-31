use froql::entity_store::{Entity, EntityGeneration, EntityId};
use quicksilver::Quicksilver;

#[expect(unused)]
pub const ENITIY_MIRROR: ::quicksilver::Type =
    ::quicksilver::Type::Struct(&::quicksilver::Struct {
        name: "Entity",
        size: ::std::mem::size_of::<Entity>(),
        align: align_of::<Entity>(),
        fields: &[
            ::quicksilver::Field {
                name: "generation",
                ty: ::quicksilver::Type::Struct(&::quicksilver::Struct {
                    name: "EntityGeneration",
                    size: ::std::mem::size_of::<EntityGeneration>(),
                    align: align_of::<EntityGeneration>(),
                    fields: &[::quicksilver::Field {
                        name: "0",
                        ty: u32::MIRROR,
                        offset: ::std::mem::offset_of!(EntityGeneration, 0),
                    }],
                }),
                offset: ::std::mem::offset_of!(Entity, generation),
            },
            ::quicksilver::Field {
                name: "id",
                ty: ::quicksilver::Type::Struct(&::quicksilver::Struct {
                    name: "EntityId",
                    size: std::mem::size_of::<EntityId>(),
                    align: align_of::<EntityId>(),
                    fields: &[::quicksilver::Field {
                        name: "0",
                        ty: u32::MIRROR,
                        offset: ::std::mem::offset_of!(EntityId, 0),
                    }],
                }),
                offset: ::std::mem::offset_of!(Entity, id),
            },
        ],
    });
