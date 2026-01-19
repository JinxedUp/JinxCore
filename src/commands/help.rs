use pumpkin::command::{
    args::{Arg, ConsumedArgs},
    CommandExecutor, CommandResult, CommandSender,
};
use pumpkin::server::Server;
use pumpkin_util::text::{color::NamedColor, TextComponent};

use crate::branding;

const ARG_PAGE: &str = "page";
const PAGE_COUNT: u32 = 11;
const COMMAND_COUNT: u32 = 53;

pub struct HelpExecutor;

impl CommandExecutor for HelpExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let page = match _args.get(ARG_PAGE) {
                Some(Arg::Simple(value)) => value.parse::<u32>().unwrap_or(1),
                _ => 1,
            };
            let page = if (1..=PAGE_COUNT).contains(&page) { page } else { 1 };

            let header = TextComponent::text(format!(
                "{COMMAND_COUNT} commands loaded.\nMade by Jinx, with a lot of love <3\n"
            ))
            .color_named(NamedColor::Gray);
            let body = match page {
                1 => TextComponent::text(
                    "Commands (1/11):\n\
/tps\n\
/uptime\n\
/seen <player>\n\
/whois <player>\n\
/clearinv [player]",
                )
                .color_named(NamedColor::White),
                2 => TextComponent::text(
                    "Commands (2/11):\n\
/rules\n\
/discord\n\
/website\n\
/store\n\
/socials",
                )
                .color_named(NamedColor::White),
                3 => TextComponent::text(
                    "Commands (3/11):\n\
/jinx reload\n\
/jinx health\n\
/jinx credits\n\
/jinx help <page>\n\
/coords",
                )
                .color_named(NamedColor::White),
                4 => TextComponent::text(
                    "Commands (4/11):\n\
/gmc [player]\n\
/gms [player]\n\
/gmsp [player]\n\
/gma [player]\n\
/creative [player]",
                )
                .color_named(NamedColor::White),
                5 => TextComponent::text(
                    "Commands (5/11):\n\
/survival [player]\n\
/spectator [player]\n\
/adventure [player]\n\
/s [player]\n\
/c [player]",
                )
                .color_named(NamedColor::White),
                6 => TextComponent::text(
                    "Commands (6/11):\n\
/a [player]\n\
/sp [player]\n\
/heal [player]\n\
/feed [player]\n\
/fly [player]",
                )
                .color_named(NamedColor::White),
                7 => TextComponent::text(
                    "Commands (7/11):\n\
/god [player]\n\
/speed <walk|fly> <value> [player]\n\
/suicide\n\
/ping [player]\n\
/near",
                )
                .color_named(NamedColor::White),
                8 => TextComponent::text(
                    "Commands (8/11):\n\
/playtime [player]\n\
/me\n\
/clearchat\n\
/createkit <name> <delay>\n\
/kit <name>",
                )
                .color_named(NamedColor::White),
                9 => TextComponent::text(
                    "Commands (9/11):\n\
/day\n\
/night\n\
/rain\n\
/clear\n\
/thunder",
                )
                .color_named(NamedColor::White),
                10 => TextComponent::text(
                    "Commands (10/11):\n\
/calc <expression>\n\
/online\n\
/flip\n\
/whoami\n\
/pl",
                )
                .color_named(NamedColor::White),
                _ => TextComponent::text(
                    "Commands (11/11):\n\
/i <targets> <item> [count]\n\
/starterkit\n\
/delstarterkit",
                )
                .color_named(NamedColor::White),
            };
            let body = header.add_child(body);
            sender.send_message(branding::brand(body)).await;
            Ok(())
        })
    }
}

pub fn jinx_help_command() -> HelpExecutor {
    HelpExecutor
}
