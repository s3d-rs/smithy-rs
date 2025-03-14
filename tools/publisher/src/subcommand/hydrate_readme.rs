/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::repo::Repository;
use crate::SMITHYRS_REPO_NAME;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use semver::Version;
use serde_json::json;
use std::fs;
use std::path::Path;

pub async fn subcommand_hydrate_readme(
    sdk_version: Version,
    msrv: String,
    output: &Path,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let repository = Repository::new(SMITHYRS_REPO_NAME, &cwd)?;
    let template_path = repository.root.join("aws/SDK_README.md.hb");
    let template_contents = fs::read(&template_path)
        .with_context(|| format!("Failed to read README template file at {:?}", template_path))?;
    let template_string =
        String::from_utf8(template_contents).context("README template file was invalid UTF-8")?;

    let hydrated = hydrate_template(&template_string, sdk_version, &msrv)?;
    fs::write(output, hydrated.as_bytes())
        .with_context(|| format!("Failed to write hydrated README to {:?}", output))?;
    Ok(())
}

fn hydrate_template(template_string: &str, sdk_version: Version, msrv: &str) -> Result<String> {
    let reg = Handlebars::new();
    reg.render_template(
        template_string,
        &json!({
            "sdk_version": format!("{}", sdk_version),
            "msrv": msrv
        }),
    )
    .context("Failed to hydrate README template")
}

#[cfg(test)]
mod tests {
    use super::hydrate_template;
    use semver::Version;

    #[test]
    fn test_hydrate() {
        let hydrated = hydrate_template(
            "\
            {{!-- Not included --}}\n\
            <!-- Included -->\n\
            Some {{sdk_version}} and {{msrv}} here.\n\
            ",
            Version::new(0, 5, 1),
            "1.54",
        )
        .unwrap();
        assert_eq!(
            "\
            <!-- Included -->\n\
            Some 0.5.1 and 1.54 here.\n\
            ",
            hydrated,
        )
    }
}
