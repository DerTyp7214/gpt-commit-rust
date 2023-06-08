use colored::*;
use std::process::Command;

pub fn run_commands(commands: &Vec<Vec<String>>) {
    for c in commands {
        let mut command = Command::new(&c[0]);
        for arg in &c[1..] {
            command.arg(arg);
        }
        command.spawn().unwrap().wait().unwrap();
    }
}

pub fn parse_commands(commands: &Vec<Vec<String>>, new_lines: bool) -> String {
    return commands
        .into_iter()
        .map(|command| {
            colorize_command(command.into_iter().map(|s| s.as_str()).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>()
        .join(&format!(
            "{}{}",
            " &&".bright_black(),
            if new_lines { "\n" } else { " " }
        ));
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

    colorize_command(line.split_whitespace().collect::<Vec<_>>())
}

fn colorize_pipe_command(command: &str, new_lines: bool) -> String {
    command
        .split(" | ")
        .map(|cmd| colorize_command(cmd.split_whitespace().collect::<Vec<_>>()))
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
        .map(|cmd| colorize_command(cmd.split_whitespace().collect::<Vec<_>>()))
        .collect::<Vec<_>>()
        .join(&format!(
            "{}{}",
            " &&".bright_black(),
            if new_lines { "\n" } else { " " }
        ))
}

fn colorize_command(command: Vec<&str>) -> String {
    let mut parts = command.into_iter();
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
    let mut messages = Vec::new();

    let mut in_message = false;

    for arg in args {
        println!("{:?}", arg);
        if arg == "-m" {
            in_message = true;
        } else if in_message {
            in_message = false;
            messages.push(format!(
                "-m {}{}{}",
                "\"".bright_black(),
                replace_gitmoji_with_emoji(arg).green(),
                "\"".bright_black()
            ));
        }
    }

    messages.join(" ")
}

pub fn replace_gitmoji_with_emoji(message: &str) -> String {
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
