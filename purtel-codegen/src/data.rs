//! Codegen. Definition of parsing targets.

use darling::FromMeta;

/// Describes the data that directly corresponds to the attributes of the
/// `purtel_task`-proc-macro. This sstruct is needed as parsing target for
/// the crate "darling" which works together with "syn". The semantically
/// meaning is the description what parameters a tasks uses in what mode
/// (read or write).
// FromMeta is from "darling" crate; implements "from_list"-factory-method
#[derive(Debug, FromMeta)]
pub struct PurtelTaskAttributes {
    /// value inside `write = "param1, param2, ..."`
    #[darling(default)]
    write: Option<String>,
    /// value inside `read = "param1, param2, ..."`
    #[darling(default)]
    read: Option<String>,
}

impl PurtelTaskAttributes {

    /// Getter. Maps property `write` of `PurtelTaskAttributes` from a
    /// comma-separated string to a vector of strings.
    pub fn write_params(&self) -> Vec<String> {
        self.write.as_ref()
            .map(|s| s.clone())
            .map(|s| s.split(",").into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>()
            )
            .unwrap_or(vec![]).into_iter()
            .map(|s| s.trim().to_owned())
            .collect::<Vec<String>>()
    }

    /// Getter. Maps property `read` of `PurtelTaskAttributes`  from a
    /// comma-separated string to a vector of strings.
    pub fn read_params(&self) -> Vec<String> {
        self.read.as_ref()
            .map(|s| s.clone())
            .map(|s| s.split(",").into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>()
            )
            .unwrap_or(vec![]).into_iter()
            .map(|s| s.trim().to_owned())
            // we filter out params in read that are already in write
            // in case a developer added the same parameter to both by accident
            .filter(|s| self.write.as_ref()
                .map(|rs| !rs.contains(s)).unwrap_or(true)
            )
            .collect::<Vec<String>>()
    }
}
