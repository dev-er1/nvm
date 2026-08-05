// nvm-cli/src/parser.rs
//
//! Превращение строки в [`Command`].
use argparser::{ArgumentParser, ParseError};

use crate::{
    cmd::Command,
    error::{CLIError, CLIErrorKind},
};

pub fn parse() -> Result<Command, CLIError> {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    if raw.is_empty() {
        return Ok(Command::Help {
            dont_show_banner: false,
            cmd: None,
        });
    }

    let command_name = raw[0].as_str();

    match command_name {
        "help" => parse_help(&raw),
        "run" => parse_run(&raw),
        "version" => Ok(Command::Version),
        cmd => Err(CLIError::new(
            CLIErrorKind::UnknownCommand(cmd.to_string()),
            Some(raw),
            Some(vec![0]),
        )),
    }
}

fn parse_help(args: &[String]) -> Result<Command, CLIError> {
    let matches = ArgumentParser::new()
        .flag_with_value("info", &["--info"])
        .flag("dont-show-banner", &["--dont-show-banner"])
        .parse(argparser::str::Source::from_iter(args.iter().cloned()));

    if let Some(err) = matches.errors.clone().first() {
        return Err(CLIError::new(
            error_kind(err),
            Some(args.to_vec()),
            // Для ошибочного флага/значения ищем индекс аргумента.
            // Если найти не удалось — пропускаем указатель (`None`).
            find_arg_index(args, offending_token(err)).map(|index| vec![index]),
        ));
    }
    Ok(Command::Help {
        dont_show_banner: matches.flag("dont-show-banner"),
        cmd: matches.value("info").map(str::to_owned),
    })
}

fn parse_run(args: &[String]) -> Result<Command, CLIError> {
    let matches = ArgumentParser::new()
        .typed_value::<usize>("memory", &["--memory"])
        .flag("time", &["--time"])
        .parse(argparser::str::Source::from_iter(args.iter().cloned()));

    if let Some(err) = matches.errors.clone().first() {
        return Err(CLIError::new(
            error_kind(err),
            Some(args.to_vec()),
            find_arg_index(args, offending_token(err)).map(|index| vec![index]),
        ));
    }

    // `positional()[0]` — всегда имя команды ("run"), т.к. парсим полные
    // аргументы. Файл, таким образом, находится по индексу `1`.
    let file = matches.get(1).ok_or_else(|| {
        CLIError::new(
            CLIErrorKind::MissingValueForCommand("run".to_string()),
            Some(args.to_vec()),
            Some(vec![0]),
        )
    })?;

    if let Some(extra) = matches.get(2) {
        return Err(CLIError::new(
            CLIErrorKind::UnexpectedArgument(extra.to_string()),
            Some(args.to_vec()),
            find_arg_index(args, extra).map(|index| vec![index]),
        ));
    }

    Ok(Command::Run {
        file: file.to_string(),
        time: matches.flag("time"),
        memory: matches.get_one::<usize>("memory").copied(),
    })
}

fn error_kind(err: &ParseError) -> CLIErrorKind {
    match err {
        ParseError::UnknownFlag(flag) => CLIErrorKind::UnknownFlag(flag.clone()),
        ParseError::UnexpectedValue { flag, value } => {
            CLIErrorKind::UnexpectedValue(flag.clone(), value.clone())
        }
        ParseError::MissingValue(flag) => CLIErrorKind::MissingValueForFlag(flag.clone()),
        ParseError::InvalidValue { flag, .. } => CLIErrorKind::InvalidValue(flag.clone()),
    }
}

/// Возвращает "токен", на который нужно указать в исходной командной строке.
///
/// `argparser` в ошибках `MissingValue`/`UnexpectedValue` отдаёт имя флага
/// без ведущих дефисов, поэтому собираем токен во всех возможных формах.
fn offending_token(err: &ParseError) -> &str {
    match err {
        ParseError::UnknownFlag(flag) => flag,
        ParseError::UnexpectedValue { flag, .. } => flag,
        ParseError::MissingValue(flag) => flag,
        ParseError::InvalidValue { flag, .. } => flag,
    }
}

/// Ищет индекс аргумента, соответствующего флагу или значению из ошибки.
///
/// Сравнение идёт по "нормализованному" имени: без ведущих дефисов
/// и без части `=...` (для флагов в форме `--flag=value`).
///
/// Например, для флага `info` подойдут аргументы `info`, `--info`,
/// `-info` и `--info=value`.
fn find_arg_index(args: &[String], needle: &str) -> Option<usize> {
    let needle = normalize(needle);

    args.iter().position(|arg| normalize(arg) == needle)
}

fn normalize(arg: &str) -> &str {
    let arg = arg.trim_start_matches('-');
    match arg.split_once('=') {
        Some((name, _)) => name,
        None => arg,
    }
}
