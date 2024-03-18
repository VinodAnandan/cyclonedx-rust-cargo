/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::external_models::normalized_string::{validate_normalized_string, NormalizedString};
use crate::models::hash::Hashes;
use crate::validation::{Validate, ValidationContext, ValidationResult};

use super::bom::SpecVersion;
use super::component::Components;
use super::service::Services;

/// Defines the creation tool(s)
///
/// In version 1.5 the type of this property changed to
/// https://cyclonedx.org/docs/1.5/json/#metadata_tools_oneOf_i0_services .
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tools {
    /// Legacy https://cyclonedx.org/docs/1.4/json/#metadata_tools
    Tools(Vec<Tool>),
    Components(Components),
    Services(Services),
}

impl Validate for Tools {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();

        if version <= SpecVersion::V1_4 {
            if !matches!(self, Tools::Tools(_)) {
                return ValidationContext::new()
                    .add_custom("inner", "Unsupported tools type found.")
                    .into();
            }
        }

        match self {
            Tools::Tools(tools) => {
                context.add_list("inner", tools, |tool| tool.validate_version(version));
            }
            Tools::Components(components) => {
                context.add_list("inner", &components.0, |component| {
                    component.validate_version(version)
                });
            }
            Tools::Services(services) => {
                context.add_list("inner", &services.0, |service| {
                    service.validate_version(version)
                });
            }
        }

        context.into()
    }
}

/// Represents the tool used to create the BOM
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_toolType)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tool {
    pub vendor: Option<NormalizedString>,
    pub name: Option<NormalizedString>,
    pub version: Option<NormalizedString>,
    pub hashes: Option<Hashes>,
}

impl Tool {
    /// Construct a `Tool` with the vendor, name, and version
    /// ```
    /// use cyclonedx_bom::models::tool::Tool;
    ///
    /// let tool = Tool::new("CycloneDX", "cargo-cyclonedx", "1.0.0");
    /// ```
    pub fn new(vendor: &str, name: &str, version: &str) -> Self {
        Self {
            vendor: Some(NormalizedString::new(vendor)),
            name: Some(NormalizedString::new(name)),
            version: Some(NormalizedString::new(version)),
            hashes: None,
        }
    }
}

impl Validate for Tool {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("vendor", self.vendor.as_ref(), validate_normalized_string)
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option("version", self.version.as_ref(), validate_normalized_string)
            .add_list("hashes", &self.hashes, |hashes| {
                hashes.validate_version(version)
            })
            .into()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        models::{
            bom::SpecVersion,
            service::Services,
            tool::{Tool, Tools},
        },
        prelude::{Components, NormalizedString, Validate},
        validation,
    };

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Tools::Tools(vec![Tool {
            vendor: Some(NormalizedString("no_whitespace".to_string())),
            name: None,
            version: None,
            hashes: None,
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Tools::Tools(vec![Tool {
            vendor: Some(NormalizedString("spaces and\ttabs".to_string())),
            name: None,
            version: None,
            hashes: None,
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::field(
                        "vendor",
                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                    )
                )]
            )
        );
    }

    #[test]
    fn it_should_merge_validations_correctly() {
        let validation_result = Tools::Tools(vec![
            Tool {
                vendor: Some(NormalizedString("no_whitespace".to_string())),
                name: None,
                version: None,
                hashes: None,
            },
            Tool {
                vendor: Some(NormalizedString("spaces and\ttabs".to_string())),
                name: None,
                version: None,
                hashes: None,
            },
            Tool {
                vendor: None,
                name: Some(NormalizedString("spaces and\ttabs".to_string())),
                version: None,
                hashes: None,
            },
        ])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [
                    (
                        1,
                        validation::field(
                            "vendor",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        )
                    ),
                    (
                        2,
                        validation::field(
                            "name",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        )
                    )
                ]
            )
        );
    }

    #[test]
    fn it_should_handle_different_tools() {
        assert!(!Tools::Services(Services(vec![]))
            .validate_version(SpecVersion::V1_3)
            .passed());
        assert!(!Tools::Services(Services(vec![]))
            .validate_version(SpecVersion::V1_4)
            .passed());
        assert!(Tools::Services(Services(vec![]))
            .validate_version(SpecVersion::V1_5)
            .passed());
        assert!(Tools::Components(Components(vec![]))
            .validate_version(SpecVersion::V1_5)
            .passed());
    }
}
