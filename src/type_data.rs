use bevy::color::Color;
use bevy::prelude::Component;
use bevy::reflect::TypeInfo;
use bevy::reflect::func::args::Ownership;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Component)]
pub struct TypeData {
    pub type_info: TypeInfo,
    pub ownership: Ownership,
}

impl Display for TypeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.ownership {
            Ownership::Ref => f.write_str("&")?,
            Ownership::Mut => f.write_str("&mut ")?,
            Ownership::Owned => {}
        };
        f.write_str(self.type_info.type_path())?;
        Ok(())
    }
}

impl Into<Color> for TypeData {
    fn into(self) -> Color {
        let mut hasher = DefaultHasher::new();
        self.to_string().hash(&mut hasher);
        let awa = (Box::new(hasher) as Box<dyn Hasher>).finish();
        let (r, g, b) = split_u64_to_u8s(awa);
        return Color::srgb_u8(r, g, b);
        pub fn split_u64_to_u8s(value: u64) -> (u8, u8, u8) {
            // Extract different parts of the u64
            // We'll take the lower bits, middle bits, and upper bits

            // Extract lower 8 bits (0-7)
            let mut first_u8 = value as u8; // This automatically wraps

            // Extract middle bits (24-31)
            let mut second_u8 = (value >> 10) as u8; // Shift and mask, then wrap

            // Extract upper bits (56-63)
            let mut third_u8 = (value >> 54) as u8; // Shift and mask, then wrap

            match third_u8 % 3 {
                0 => {
                    while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                        first_u8 = first_u8.saturating_add(1);
                    }
                }
                1 => {
                    while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                        second_u8 = second_u8.saturating_add(1);
                    }
                }
                _ => {
                    while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                        third_u8 = third_u8.saturating_add(1);
                    }
                }
            }

            (first_u8, second_u8, third_u8)
        }
    }
}
