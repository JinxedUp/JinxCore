use pumpkin::command::{
    args::{ConsumedArgs, message::MsgArgConsumer},
    CommandExecutor, CommandResult, CommandSender, tree::CommandTree,
    tree::builder::argument,
};
use pumpkin::server::Server;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::branding;

const ARG_EXPR: &str = "expr";

struct CalcExecutor;

impl CommandExecutor for CalcExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(pumpkin::command::args::Arg::Msg(expr)) = args.get(ARG_EXPR) else {
                return Ok(());
            };
            let cleaned = normalize_expression(expr);
            match meval::eval_str(&cleaned) {
                Ok(value) => {
                    let msg = branding::brand(
                        TextComponent::text(format!("Result: {value}"))
                            .color_named(NamedColor::Green),
                    );
                    sender.send_message(msg).await;
                }
                Err(err) => {
                    let msg = branding::brand(
                        TextComponent::text(format!("Invalid expression: {err}"))
                            .color_named(NamedColor::Red),
                    );
                    sender.send_message(msg).await;
                }
            }
            Ok(())
        })
    }
}

fn normalize_expression(expr: &str) -> String {
    let mut out = expr.trim().to_string();
    out = out.replace(" x ", " * ");
    out = out.replace('×', "*");
    out = out.replace('X', "*");
    out
}

pub fn calc_command_tree() -> CommandTree {
    CommandTree::new(["calc"], "Evaluate a math expression.")
        .then(argument(ARG_EXPR, MsgArgConsumer).execute(CalcExecutor))
}
