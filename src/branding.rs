use pumpkin_util::text::{TextComponent, color::NamedColor};

pub fn prefix() -> TextComponent {
    TextComponent::text("JinxCore")
        .color_named(NamedColor::Gold)
        .bold()
        .add_child(TextComponent::text(" » ").color_named(NamedColor::DarkGray))
}

pub fn brand(message: TextComponent) -> TextComponent {
    prefix().add_child(message)
}
