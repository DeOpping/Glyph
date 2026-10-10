use std::sync::Arc;

use pumpkin_plugin_api::{
    Context, Server,
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    permission::{Permission, PermissionDefault, PermissionLevel},
    text::TextComponent,
};

use crate::{
    expansion::remote_registry::RemoteExpansionRegistry,
    placeholder::{PlaceholderApi, PlaceholderContext, PlayerContext},
};

use super::ipc::parse_with_registries;

const PARSE_PERMISSION: &str = "Glyph:parse";

fn get_string_argument(args: &ConsumedArgs, name: &str) -> Option<String> {
    match args.get_value(name) {
        Arg::Simple(value) | Arg::Msg(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn register(
    context: &Context,
    placeholders: Arc<PlaceholderApi<PlaceholderContext>>,
    remote_expansions: Arc<RemoteExpansionRegistry>,
) -> pumpkin_plugin_api::Result<()> {
    context.register_permission(&Permission {
        node: PARSE_PERMISSION.to_owned(),
        description: "Allows parsing Glyph placeholders for a player.".to_owned(),
        default: PermissionDefault::Op(PermissionLevel::Two),
        children: Vec::new(),
    })?;

    let names = ["glyph".to_owned()];

    let command = Command::new(&names, "Glyph placeholder commands").then(
        CommandNode::literal("parse").then(
            CommandNode::argument("target", &ArgumentType::String(StringType::SingleWord)).then(
                CommandNode::argument("message", &ArgumentType::String(StringType::Greedy))
                    .execute(ParseCommand {
                        placeholders,
                        remote_expansions,
                    }),
            ),
        ),
    );

    context.register_command(command, PARSE_PERMISSION);

    Ok(())
}

struct ParseCommand {
    placeholders: Arc<PlaceholderApi<PlaceholderContext>>,
    remote_expansions: Arc<RemoteExpansionRegistry>,
}

impl CommandHandler for ParseCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let Some(target) = get_string_argument(&args, "target") else {
            sender.send_message(TextComponent::text(
                "Missing target. Usage: /glyph parse <player|me> <text...>",
            ));

            return Ok(0);
        };

        let Some(message) = get_string_argument(&args, "message") else {
            sender.send_message(TextComponent::text(
                "Missing message. Usage: /glyph parse <player|me> <text...>",
            ));

            return Ok(0);
        };

        if message.trim().is_empty() {
            sender.send_message(TextComponent::text("Please provide text to parse."));

            return Ok(0);
        }

        let player = if target.eq_ignore_ascii_case("me") {
            match sender.as_player() {
                Some(player) => player,
                None => {
                    sender.send_message(TextComponent::text(
                        "'me' can only be used by a player. Specify an online player instead.",
                    ));

                    return Ok(0);
                }
            }
        } else {
            match server.get_player_by_name(&target) {
                Some(player) => player,
                None => {
                    let message = format!("Player '{target}' is not online.");

                    sender.send_message(TextComponent::text(&message));

                    return Ok(0);
                }
            }
        };

        let context = PlaceholderContext {
            player: Some(PlayerContext {
                uuid: Some(player.get_id().to_string()),
                name: player.get_name(),
                locale: Some(player.get_locale()),
            }),
            values: Default::default(),
        };

        let result = parse_with_registries(
            &self.placeholders,
            &self.remote_expansions,
            message.trim_start(),
            &context,
        );

        sender.send_message(TextComponent::text(&result));

        Ok(1)
    }
}
