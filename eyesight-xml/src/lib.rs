use enum_dispatch::enum_dispatch;

macro_rules! enums {
    (
        $(
            $enum:ident {
                $($variant:ident),*$(,)?
            }
        )*
    ) => {
        $(
            #[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
            #[serde(rename_all = "snake_case")]
            pub enum $enum {
                $($variant),*
            }
        )*
    }
}

#[enum_dispatch]
pub trait Named {
    fn name(&self) -> &str;
    fn name_mut(&mut self) -> &mut String;
}

pub mod nodes;
pub mod schema;
