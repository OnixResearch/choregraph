use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceBranch {
    pub label: super::participants::LabelId,
    pub body: Node,
    pub span: super::participants::SourceSpan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    rename = "ChoreographyNode",
    tag = "kind",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum Node {
    End {
        span: super::participants::SourceSpan,
    },
    Communication {
        from: super::participants::RoleId,
        to: super::participants::RoleId,
        label: super::participants::LabelId,
        payload_tag: super::participants::PayloadTag,
        next: Box<Node>,
        span: super::participants::SourceSpan,
    },
    Transfer {
        from: super::participants::RoleId,
        to: super::participants::RoleId,
        label: super::participants::LabelId,
        payload_tag: super::participants::PayloadTag,
        value: super::participants::ValueId,
        next: Box<Node>,
        span: super::participants::SourceSpan,
    },
    Choice {
        decider: super::participants::RoleId,
        branches: Vec<ChoiceBranch>,
        span: super::participants::SourceSpan,
    },
}

impl Node {
    #[must_use]
    pub const fn span(&self) -> Option<super::participants::SourceSpan> {
        match self {
            Self::End { span } | Self::Communication { span, .. } | Self::Transfer { span, .. } => {
                Some(*span)
            }
            other => {
                if let Self::Choice { span, .. } = other {
                    Some(*span)
                } else {
                    None
                }
            }
        }
    }
}
