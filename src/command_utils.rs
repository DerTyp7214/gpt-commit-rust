use colored::*;
use std::process::{Child, Command};

pub fn run_commands(commands: &str) {
    if commands.contains(" && ") {
        return run_and_commands(commands);
    }

    if commands.contains(" | ") {
        return run_pipe_commands(commands);
    }

    let commands = commands.split('\n');
    for c in commands {
        if c.trim() == "" {
            continue;
        }
        run_command(c).unwrap().wait().unwrap();
    }
}

fn run_and_commands(commands: &str) {
    let commands = commands.split(" && ");
    let mut last_command = None;
    for c in commands {
        if c.trim() == "" {
            continue;
        }
        let mut command = run_command(c).unwrap();
        command.wait().unwrap();
        last_command = Some(command);
    }
    if let Some(mut command) = last_command {
        command.wait().unwrap();
    }
}

fn run_pipe_commands(commands: &str) {
    let commands = commands.split(" | ");
    let mut last_command: Option<Child> = None;
    for c in commands {
        if c.trim() == "" {
            continue;
        }
        let mut command = run_command(c).unwrap();
        if let Some(mut last_command) = last_command {
            last_command.stdout = command.stdout.take();
            last_command.wait().unwrap();
        }
        last_command = Some(command);
    }
    if let Some(mut command) = last_command {
        command.wait().unwrap();
    }
}

fn run_command(command: &str) -> Result<Child, std::io::Error> {
    let mut parts = command.split_whitespace();
    let cmd = parts.next().unwrap();
    let args = parts.collect::<Vec<_>>();
    let args = args
        .into_iter()
        .fold::<Vec<String>, _>(vec![], |mut args, arg| match args.last_mut() {
            Some(last) if last.starts_with('"') => {
                *last = format!("{} {}", last, arg);
                args
            }
            _ => {
                args.push(arg.to_owned());
                args
            }
        });
    Command::new(cmd).args(args).spawn()
}

pub fn parse_command(line: &str, new_lines: bool) -> String {
    if line.contains('\n') {
        return line
            .split('\n')
            .map(|line| parse_command(line, new_lines))
            .collect::<Vec<_>>()
            .join("\n");
    }

    if line.contains(" | ") {
        return colorize_pipe_command(line, new_lines);
    }

    if line.contains(" && ") {
        return colorize_and_command(line, new_lines);
    }

    colorize_command(line)
}

fn colorize_pipe_command(command: &str, new_lines: bool) -> String {
    command
        .split(" | ")
        .map(|cmd| colorize_command(cmd))
        .collect::<Vec<_>>()
        .join(&format!(
            "{}{}",
            " |".bright_black(),
            if new_lines { "\n" } else { " " }
        ))
}

fn colorize_and_command(command: &str, new_lines: bool) -> String {
    command
        .split(" && ")
        .map(|cmd| colorize_command(cmd))
        .collect::<Vec<_>>()
        .join(&format!(
            "{}{}",
            " &&".bright_black(),
            if new_lines { "\n" } else { " " }
        ))
}

fn colorize_command(command: &str) -> String {
    let mut parts = command.split_whitespace();
    if let Some(cmd) = parts.next() {
        if cmd == "git" {
            return format!(
                "{} {}",
                cmd.yellow(),
                colorize_git_command(parts.collect::<Vec<_>>())
            );
        }
        return format!(
            "{} {}",
            cmd.yellow(),
            parts
                .into_iter()
                .map(|part| {
                    if part.starts_with("-") {
                        part.bright_blue().to_string()
                    } else {
                        part.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    String::new()
}

fn colorize_git_command(args: Vec<&str>) -> String {
    if let Some((cmd, rest)) = args.split_first() {
        if cmd == &"commit" {
            return format!("{} {}", cmd, colorize_git_commit_command(rest.to_vec()));
        }
        return format!(
            "{} {}",
            cmd,
            rest.iter()
                .map(|arg| {
                    if arg.starts_with('-') {
                        arg.bright_blue().to_string()
                    } else {
                        arg.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    String::new()
}

fn colorize_git_commit_command(args: Vec<&str>) -> String {
    if let Some((cmd, rest)) = args.split_first() {
        if cmd == &"-m" || cmd == &"--message" || cmd == &"-am" {
            return format!(
                "{} {}",
                cmd.bright_blue(),
                colorize_git_commit_message_command(rest.to_vec())
            );
        }

        if cmd.starts_with('-') {
            return format!("{} {}", cmd.bright_blue(), rest.join(" "));
        }

        return format!("{} {}", cmd.magenta(), rest.join(" "));
    }
    String::new()
}

fn colorize_git_commit_message_command(args: Vec<&str>) -> String {
    let message = args.join(" ");
    if let (Some(message_start), Some(message_end)) = (message.find('"'), message.rfind('"')) {
        let message_content = &message[message_start + 1..message_end];
        let message_colorized = replace_gitmoji_with_emoji(message_content).green();
        let message_colorized_full = format!(
            "{}{}{}",
            &message[..message_start + 1],
            message_colorized,
            &message[message_end..]
        );
        return message_colorized_full;
    }
    String::new()
}

fn replace_gitmoji_with_emoji(message: &str) -> String {
    let gitmoji_regex = regex::Regex::new(r":[a-z0-9_]+:").unwrap();
    let mut replaced_message = message.to_owned();

    for gitmoji in gitmoji_regex.find_iter(message) {
        let emoji = gitmoji.as_str().trim_matches(':');
        if let Some(emoji_replacement) = EMOJI_MAP.get(emoji) {
            replaced_message = replaced_message.replace(gitmoji.as_str(), emoji_replacement);
        }
    }

    replaced_message
}

lazy_static::lazy_static! {
    static ref EMOJI_MAP: std::collections::HashMap<&'static str, &'static str> = {
        let mut map = std::collections::HashMap::new();
        map.insert("art", "🎨");
        map.insert("zap", "⚡️");
        map.insert("fire", "🔥");
        map.insert("bug", "🐛");
        map.insert("ambulance", "🚑");
        map.insert("sparkles", "✨");
        map.insert("memo", "📝");
        map.insert("rocket", "🚀");
        map.insert("lipstick", "💄");
        map.insert("tada", "🎉");
        map.insert("white_check_mark", "✅");
        map.insert("lock", "🔒");
        map.insert("closed_lock_with_key", "🔐");
        map.insert("bookmark", "🔖");
        map.insert("rotating_light", "🚨");
        map.insert("construction", "🚧");
        map.insert("green_heart", "💚");
        map.insert("arrow_down", "⬇️");
        map.insert("arrow_up", "⬆️");
        map.insert("pushpin", "📌");
        map.insert("construction_worker", "👷");
        map.insert("chart_with_upwards_trend", "📈");
        map.insert("recycle", "♻️");
        map.insert("heavy_plus_sign", "➕");
        map.insert("heavy_minus_sign", "➖");
        map.insert("wrench", "🔧");
        map.insert("hammer", "🔨");
        map.insert("globe_with_meridians", "🌐");
        map.insert("pencil2", "✏️");
        map.insert("pencil", "✏️");
        map.insert("poop", "💩");
        map.insert("rewind", "⏪");
        map.insert("twisted_rightwards_arrows", "🔀");
        map.insert("package", "📦");
        map.insert("alien", "👽");
        map.insert("truck", "🚚");
        map.insert("page_facing_up", "📄");
        map.insert("boom", "💥");
        map.insert("bento", "🍱");
        map.insert("wheelchair", "♿️");
        map.insert("bulb", "💡");
        map.insert("beers", "🍻");
        map.insert("speech_balloon", "💬");
        map.insert("card_file_box", "🗃️");
        map.insert("loud_sound", "🔊");
        map.insert("mute", "🔇");
        map.insert("busts_in_silhouette", "👥");
        map.insert("children_crossing", "🚸");
        map.insert("building_construction", "🏗️");
        map.insert("iphone", "📱");
        map.insert("clown_face", "🤡");
        map.insert("egg", "🥚");
        map.insert("see_no_evil", "🙈");
        map.insert("camera_flash", "📸");
        map.insert("alembic", "⚗️");
        map.insert("mag", "🔍");
        map.insert("label", "🏷️");
        map.insert("seedling", "🌱");
        map.insert("triangular_flag_on_post", "🚩");
        map.insert("goal_net", "🥅");
        map.insert("dizzy", "💫");
        map.insert("wastebasket", "🗑️");
        map.insert("passport_control", "🛂");
        map.insert("adhesive_bandage", "🩹");
        map.insert("monocle_face", "🧐");
        map.insert("coffin", "⚰️");
        map.insert("test_tube", "🧪");
        map.insert("necktie", "👔");
        map.insert("stethoscope", "🩺");
        map.insert("bricks", "🧱");
        map.insert("technologist", "🧑‍💻");
        map.insert("money_with_wings", "💸");
        map.insert("thread", "🧵");
        map.insert("safety_vest", "🦺");
        map
    };
}
