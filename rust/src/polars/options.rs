use polars::prelude::{
    JoinArgs, JoinCoalesce, JoinOptions, JoinType, JoinValidation, MaintainOrderJoin,
};
use polars_plan::prelude::{GroupbyOptions, ProjectionOptions};

// Enumerate fields explicitly so new Polars options require a compile-time review.
pub(crate) const ORDINARY_GROUP_BY_OPTIONS: GroupbyOptions = GroupbyOptions {
    dynamic: None,
    rolling: None,
    slice: None,
};

// No '..': new options cannot silently enter the accepted set.
pub(crate) fn valid_projection_options(options: ProjectionOptions) -> bool {
    matches!(
        options,
        ProjectionOptions {
            run_parallel: _,
            duplicate_check: true,
            should_broadcast: true,
        }
    )
}

// The caller separately validates key expressions, coalescing, and output schemas.
pub(crate) fn valid_public_key_join_options(options: &JoinOptions) -> bool {
    matches!(
        options,
        JoinOptions {
            allow_parallel: _,
            force_parallel: _,
            args: JoinArgs {
                how: JoinType::Left | JoinType::Right,
                validation: JoinValidation::ManyToMany,
                suffix: _,
                slice: None,
                nulls_equal: _,
                coalesce: JoinCoalesce::JoinSpecific | JoinCoalesce::CoalesceColumns,
                maintain_order: MaintainOrderJoin::None,
            },
        }
    ) && matches!(options.args.suffix.as_deref(), None | Some("_right"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_options() {
        let good = ProjectionOptions::default();
        assert!(valid_projection_options(good));
        assert!(valid_projection_options(ProjectionOptions {
            run_parallel: false,
            ..good
        }));
        assert!(!valid_projection_options(ProjectionOptions {
            duplicate_check: false,
            ..good
        }));
        assert!(!valid_projection_options(ProjectionOptions {
            should_broadcast: false,
            ..good
        }));
    }

    #[test]
    fn public_key_join_options() {
        let mut options = JoinOptions {
            allow_parallel: true,
            force_parallel: true,
            args: JoinArgs {
                how: JoinType::Right,
                validation: JoinValidation::ManyToMany,
                suffix: None,
                slice: None,
                nulls_equal: true,
                coalesce: JoinCoalesce::JoinSpecific,
                maintain_order: MaintainOrderJoin::None,
            },
        };
        assert!(valid_public_key_join_options(&options));
        options.allow_parallel = false;
        options.force_parallel = false;
        options.args.coalesce = JoinCoalesce::CoalesceColumns;
        assert!(valid_public_key_join_options(&options));
        options.args.coalesce = JoinCoalesce::KeepColumns;
        assert!(!valid_public_key_join_options(&options));
        options.args.coalesce = JoinCoalesce::JoinSpecific;
        options.args.slice = Some((0, 1));
        assert!(!valid_public_key_join_options(&options));
        options.args.slice = None;
        options.args.maintain_order = MaintainOrderJoin::Left;
        assert!(!valid_public_key_join_options(&options));
        options.args.maintain_order = MaintainOrderJoin::None;
        options.args.suffix = Some("unapproved".into());
        assert!(!valid_public_key_join_options(&options));
    }
}
