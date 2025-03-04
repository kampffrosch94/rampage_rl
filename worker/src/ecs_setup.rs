use nanoserde::{DeJson, SerJson};
use std::collections::HashMap;

#[derive(Default, Debug, DeJson, SerJson)]
pub struct SerializedState {
    // TypeName, Vec<(EntityId, ComponentPayload)>
    pub components: HashMap<String, Vec<(u32, String)>>,
    // TypeName, Vec<(Origin, Target)>
    pub relations: HashMap<String, Vec<(u32, u32)>>,
}

macro_rules! generate_register {
    (@rel $world:ident $ty:tt $flags:tt) => {
        $world.register_relation_flags::<$ty>($flags);
    };
    (@rel $world:ident $ty:tt) => {
        $world.register_relation::<$ty>();
    };
    (Components($($components:tt $([persist])?),*),
     Relations($($relations:tt $(($flags:expr))? $([persist])? ),*)) => {
        pub fn register_components(world: &mut World) {
            $(world.register_component::<$components>();)*
            $(generate_register!(@rel world $relations $($flags)?);)*
        }
    };
}

macro_rules! generate_re_register {
    (@rel $world:ident $ty:tt $flags:tt) => {
        $world.re_register_relation::<$ty>()?;
    };
    (@rel $world:ident $ty:tt) => {
        $world.re_register_relation::<$ty>()?;
    };
    (Components($($components:tt $([persist])?),*),
     Relations($($relations:tt $(($flags:expr))? $([persist])? ),*)) => {
        pub fn re_register_components(world: &mut World) -> Result<(), ()> {
            unsafe {
                $(world.re_register_component::<$components>()?;)*
                $(generate_register!(@rel world $relations $($flags)?);)*
            }
            Ok(())
        }
    };
}

macro_rules! generate_save {
    (@rel $world:ident $state:ident $ty:tt persist) => {
        $state.relations.insert(
            type_name::<$ty>().to_string(),
            $world
                .bookkeeping
                .relation_pairs(TypeId::of::<Relation<$ty>>())
                .into_iter()
                .map(|(o, t)| (o.id.0, t.id.0))
                .collect(),
        );
    };
    (@rel $world:ident $state:ident $ty:tt ) => {};
    (@comp $world:ident $state:ident $ty:tt persist) => {
        let mut buffer = Vec::new();
        for id in trivial_query_one_component($world, TypeId::of::<RefCell<$ty>>()) {
            let r = $world.get_component_by_entityid::<$ty>(id);
            let s = r.serialize_json();
            buffer.push((id.0, s));
        }
        $state
            .components
            .insert(type_name::<$ty>().to_string(), buffer);
    };
    (@comp $world:ident $state:ident $ty:tt) => {};
    (Components($($components:tt $([$persist_comp:tt])?),*),
     Relations($($relations:tt $(($flags:expr))? $([$persist_rel:tt])?),*)) => {
        pub fn save_world(world: &World) -> String {
            let mut state = SerializedState::default();
            $(generate_save!(@comp world state $components $($persist_comp)?);)*
            $(generate_save!(@rel world state $relations $($persist_rel)?);)*
            state.serialize_json()
        }
    };
}

macro_rules! generate_load {
    (@rel ($world:expr) $var:ident $pairs:ident $ty:tt persist) => {
        if $var == type_name::<$ty>() {
            for (origin, target) in $pairs {
                let a = $world.ensure_alive(EntityId(*origin));
                let b = $world.ensure_alive(EntityId(*target));
                $world.add_relation::<$ty>(a, b);
            }
            continue;
        }
    };
    (@rel ($world:expr) $var:ident $payloads:ident $ty:tt) => {};
    (@comp ($world:expr) $var:ident $payloads:ident $ty:tt persist) => {
        if $var == type_name::<$ty>() {
            for (entity_id, payload) in $payloads {
                let val = $ty::deserialize_json(payload).unwrap();
                let e = $world.ensure_alive(EntityId(*entity_id));
                $world.add_component(e, val);
            }
            continue;
        }
    };
    (@comp ($world:expr) $var:ident $payloads:ident $ty:tt) => {};
    (Components($($components:tt $([$persist_comp:tt])?),*),
     Relations($($relations:tt $(($flags:expr))? $([$persist_rel:tt])?),*)) => {
        #[allow(unused)]
        pub fn load_world(s: &str) -> World {
            let mut world = World::new();
            register_components(&mut world);
            let state: SerializedState = SerializedState::deserialize_json(s).unwrap();

            //$(generate_load!(@comp world state $components $($persist_comp)?);)*
            //$(generate_load!(@rel world state $relations $($persist_rel)?);)*

            for (ty, payloads) in &state.components {
                let var = ty.as_str();
                $(generate_load!(@comp (&mut world) var payloads $components $($persist_comp)?);)*
                panic!("Unknown component type: {var}");
            }

            for (ty, pairs) in &state.relations {
                let var = ty.as_str();
                $(generate_load!(@rel (&mut world) var pairs $relations $($persist_rel)?);)*
                panic!("Unknown relationship type: {var}");
            }

            world
        }
    };
}

macro_rules! ecs_types {
    ($($tokens:tt)+) => {
        generate_register!($($tokens)+);
        generate_re_register!($($tokens)+);
        generate_save!($($tokens)+);
        generate_load!($($tokens)+);
    }
}

pub(crate) use ecs_types;
pub(crate) use generate_load;
pub(crate) use generate_re_register;
pub(crate) use generate_register;
pub(crate) use generate_save;
