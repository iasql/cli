// Implement own theme from ColorfulTheme to fully own UX on multiselect
// Modfied:
// - checked_item_prefix
// - unchecked_item_prefix
// - active_item_prefix
// - error_style
// - prompt_prefix
// Added
// - format_multi_select_prompt_item
// - warn_prefix
use dialoguer::{
  console::{style, Style, StyledObject},
  theme::Theme,
};
use std::fmt;

/// A colorful theme
pub struct ColorfulTheme {
  /// The style for default values
  pub defaults_style: Style,
  /// The style for prompt
  pub prompt_style: Style,
  /// Prompt prefix value and style
  pub prompt_prefix: StyledObject<String>,
  /// Prompt suffix value and style
  pub prompt_suffix: StyledObject<String>,
  /// Prompt on success prefix value and style
  pub success_prefix: StyledObject<String>,
  /// Prompt on success suffix value and style
  pub success_suffix: StyledObject<String>,
  /// Warning prefix value and style
  pub warn_prefix: StyledObject<String>,
  /// Error prefix value and style
  pub error_prefix: StyledObject<String>,
  /// The style for error message
  pub error_style: Style,
  /// The style for hints
  pub hint_style: Style,
  /// The style for values on prompt success
  pub values_style: Style,
  /// The style for active items
  pub active_item_style: Style,
  /// The style for inactive items
  pub inactive_item_style: Style,
  /// Active item in select prefix value and style
  pub active_item_prefix: StyledObject<String>,
  /// Inctive item in select prefix value and style
  pub inactive_item_prefix: StyledObject<String>,
  /// Checked item in multi select prefix value and style
  pub checked_item_prefix: StyledObject<String>,
  /// Unchecked item in multi select prefix value and style
  pub unchecked_item_prefix: StyledObject<String>,
  /// Picked item in sort prefix value and style
  pub picked_item_prefix: StyledObject<String>,
  /// Unpicked item in sort prefix value and style
  pub unpicked_item_prefix: StyledObject<String>,
  /// Show the selections from certain prompts inline
  pub inline_selections: bool,
}

impl Default for ColorfulTheme {
  fn default() -> ColorfulTheme {
    ColorfulTheme {
      defaults_style: Style::new().for_stderr().cyan(),
      prompt_style: Style::new().for_stderr().bold(),
      prompt_prefix: style("?".to_string()).for_stderr().yellow().bold(),
      prompt_suffix: style("???".to_string()).for_stderr().black().bright(),
      success_prefix: style("???".to_string()).for_stderr().green(),
      success_suffix: style("??".to_string()).for_stderr().black().bright(),
      warn_prefix: style("!".to_string()).for_stderr().yellow().bold(),
      error_prefix: style("???".to_string()).for_stderr().red(),
      error_style: Style::new().for_stderr(),
      hint_style: Style::new().for_stderr().black().bright(),
      values_style: Style::new().for_stderr().green(),
      active_item_style: Style::new().for_stderr().cyan(),
      inactive_item_style: Style::new().for_stderr(),
      active_item_prefix: style("???".to_string()).for_stderr().cyan(),
      inactive_item_prefix: style(" ".to_string()).for_stderr(),
      checked_item_prefix: style("[???]".to_string()).green().for_stderr(),
      unchecked_item_prefix: style("[ ]".to_string()).for_stderr(),
      picked_item_prefix: style("???".to_string()).for_stderr().green(),
      unpicked_item_prefix: style(" ".to_string()).for_stderr(),
      inline_selections: true,
    }
  }
}

impl Theme for ColorfulTheme {
  /// Formats a prompt.
  fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.prompt_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }

    write!(f, "{}", &self.prompt_suffix)
  }

  /// Formats an error
  fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
    write!(
      f,
      "{} {}",
      &self.error_prefix,
      self.error_style.apply_to(err)
    )
  }

  /// Formats an input prompt.
  fn format_input_prompt(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    default: Option<&str>,
  ) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.prompt_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }

    match default {
      Some(default) => write!(
        f,
        "{} {} ",
        self.hint_style.apply_to(&format!("({})", default)),
        &self.prompt_suffix
      ),
      None => write!(f, "{} ", &self.prompt_suffix),
    }
  }

  /// Formats a confirm prompt.
  fn format_confirm_prompt(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    default: Option<bool>,
  ) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.prompt_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }

    match default {
      None => write!(
        f,
        "{} {}",
        self.hint_style.apply_to("(y/n)"),
        &self.prompt_suffix
      ),
      Some(true) => write!(
        f,
        "{} {} {}",
        self.hint_style.apply_to("(y/n)"),
        &self.prompt_suffix,
        self.defaults_style.apply_to("yes")
      ),
      Some(false) => write!(
        f,
        "{} {} {}",
        self.hint_style.apply_to("(y/n)"),
        &self.prompt_suffix,
        self.defaults_style.apply_to("no")
      ),
    }
  }

  /// Formats a confirm prompt after selection.
  fn format_confirm_prompt_selection(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    selection: Option<bool>,
  ) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.success_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }
    let selection = selection.map(|b| if b { "yes" } else { "no" });

    match selection {
      Some(selection) => {
        write!(
          f,
          "{} {}",
          &self.success_suffix,
          self.values_style.apply_to(selection)
        )
      }
      None => {
        write!(f, "{}", &self.success_suffix)
      }
    }
  }

  /// Formats an input prompt after selection.
  fn format_input_prompt_selection(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    sel: &str,
  ) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.success_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }

    write!(
      f,
      "{} {}",
      &self.success_suffix,
      self.values_style.apply_to(sel)
    )
  }

  /// Formats a password prompt after selection.
  fn format_password_prompt_selection(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
    self.format_input_prompt_selection(f, prompt, "********")
  }

  /// Formats a multi select prompt after selection.
  fn format_multi_select_prompt_selection(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    selections: &[&str],
  ) -> fmt::Result {
    if !prompt.is_empty() {
      write!(
        f,
        "{} {} ",
        &self.success_prefix,
        self.prompt_style.apply_to(prompt)
      )?;
    }

    write!(f, "{} ", &self.success_suffix)?;

    if self.inline_selections {
      for (idx, sel) in selections.iter().enumerate() {
        write!(
          f,
          "{}{}",
          if idx == 0 { "" } else { ", " },
          self.values_style.apply_to(sel)
        )?;
      }
    }

    Ok(())
  }

  /// Formats a select prompt item.
  fn format_select_prompt_item(
    &self,
    f: &mut dyn fmt::Write,
    text: &str,
    active: bool,
  ) -> fmt::Result {
    let details = match active {
      true => (
        &self.active_item_prefix,
        self.active_item_style.apply_to(text),
      ),
      false => (
        &self.inactive_item_prefix,
        self.inactive_item_style.apply_to(text),
      ),
    };

    write!(f, "{} {}", details.0, details.1)
  }

  /// Formats a multi select prompt item.
  fn format_multi_select_prompt_item(
    &self,
    f: &mut dyn fmt::Write,
    text: &str,
    checked: bool,
    active: bool,
  ) -> fmt::Result {
    let details = match (checked, active) {
      (true, true) => (
        &self.active_item_prefix,
        &self.checked_item_prefix,
        self.active_item_style.apply_to(text),
      ),
      (true, false) => (
        &self.inactive_item_prefix,
        &self.checked_item_prefix,
        self.inactive_item_style.apply_to(text),
      ),
      (false, true) => (
        &self.active_item_prefix,
        &self.unchecked_item_prefix,
        self.active_item_style.apply_to(text),
      ),
      (false, false) => (
        &self.inactive_item_prefix,
        &self.unchecked_item_prefix,
        self.inactive_item_style.apply_to(text),
      ),
    };

    write!(f, "{} {} {}", details.0, details.1, details.2)
  }

  /// Formats a sort prompt item.
  fn format_sort_prompt_item(
    &self,
    f: &mut dyn fmt::Write,
    text: &str,
    picked: bool,
    active: bool,
  ) -> fmt::Result {
    let details = match (picked, active) {
      (true, true) => (
        &self.picked_item_prefix,
        self.active_item_style.apply_to(text),
      ),
      (false, true) => (
        &self.unpicked_item_prefix,
        self.active_item_style.apply_to(text),
      ),
      (_, false) => (
        &self.unpicked_item_prefix,
        self.inactive_item_style.apply_to(text),
      ),
    };

    write!(f, "{} {}", details.0, details.1)
  }
}
