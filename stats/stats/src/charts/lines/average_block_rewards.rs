use std::ops::Range;

use crate::{
    data_source::kinds::{
        data_manipulation::{
            map::{MapParseTo, MapToString},
            resolutions::average::AverageLowerResolution,
        },
        local_db::{
            parameters::update::batching::parameters::{
                Batch30Days, Batch30Weeks, Batch30Years, Batch36Months,
            },
            DirectVecLocalDbChartSource,
        },
        remote_db::{PullAllWithAndSort, RemoteDatabaseSource, StatementFromRange},
    },
    define_and_impl_resolution_properties,
    types::timespans::{Month, Week, Year},
    utils::sql_with_range_filter_opt,
    ChartProperties, Named,
};

use chrono::NaiveDate;
use entity::sea_orm_active_enums::ChartType;
use sea_orm::{prelude::*, DbBackend, Statement};

use super::new_blocks::{NewBlocksInt, NewBlocksMonthlyInt};

const ETH: i64 = 1_000_000_000_000_000_000;

pub struct AverageBlockRewardsQuery;

impl StatementFromRange for AverageBlockRewardsQuery {
    fn get_statement(range: Option<Range<DateTimeUtc>>) -> Statement {
        sql_with_range_filter_opt!(
            DbBackend::Postgres,
            r#"
                SELECT
                    DATE(blocks.timestamp) as date,
                    (AVG(block_rewards.reward) / $1)::FLOAT as value
                FROM block_rewards
                JOIN blocks ON block_rewards.block_hash = blocks.hash
                WHERE
                    blocks.timestamp != to_timestamp(0) AND
                    blocks.consensus = true {filter}
                GROUP BY date
            "#,
            [ETH.into()],
            "blocks.timestamp",
            range,
        )
    }
}

pub type AverageBlockRewardsRemote =
    RemoteDatabaseSource<PullAllWithAndSort<AverageBlockRewardsQuery, NaiveDate, f64>>;

pub type AverageBlockRewardsRemoteString = MapToString<AverageBlockRewardsRemote>;

pub struct Properties;

impl Named for Properties {
    fn name() -> String {
        "averageBlockRewards".into()
    }
}

impl ChartProperties for Properties {
    type Resolution = NaiveDate;

    fn chart_type() -> ChartType {
        ChartType::Line
    }
}

define_and_impl_resolution_properties!(
    define_and_impl: {
        WeeklyProperties: Week,
        MonthlyProperties: Month,
        YearlyProperties: Year,
    },
    base_impl: Properties
);

pub type AverageBlockRewards =
    DirectVecLocalDbChartSource<AverageBlockRewardsRemoteString, Batch30Days, Properties>;
pub type AverageBlockRewardsWeekly = DirectVecLocalDbChartSource<
    MapToString<AverageLowerResolution<MapParseTo<AverageBlockRewards, f64>, NewBlocksInt, Week>>,
    Batch30Weeks,
    WeeklyProperties,
>;
pub type AverageBlockRewardsMonthly = DirectVecLocalDbChartSource<
    MapToString<AverageLowerResolution<MapParseTo<AverageBlockRewards, f64>, NewBlocksInt, Month>>,
    Batch36Months,
    MonthlyProperties,
>;
pub type AverageBlockRewardsYearly = DirectVecLocalDbChartSource<
    MapToString<
        AverageLowerResolution<
            MapParseTo<AverageBlockRewardsMonthly, f64>,
            NewBlocksMonthlyInt,
            Year,
        >,
    >,
    Batch30Years,
    YearlyProperties,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::simple_test::simple_test_chart;

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_average_block_rewards() {
        simple_test_chart::<AverageBlockRewards>(
            "update_average_block_rewards",
            vec![
                ("2022-11-09", "0"),
                ("2022-11-10", "2"),
                ("2022-11-11", "1.75"),
                ("2022-11-12", "3"),
                ("2022-12-01", "4"),
                ("2023-01-01", "0"),
                ("2023-02-01", "1"),
                ("2023-03-01", "2"),
            ],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_average_block_rewards_weekly() {
        simple_test_chart::<AverageBlockRewardsWeekly>(
            "update_average_block_rewards_weekly",
            // first week avgs and block counts (after /):
            // ("2022-11-09", "0"),     / 1
            // ("2022-11-10", "2"),     / 3
            // ("2022-11-11", "1.75"),  / 4
            // ("2022-11-12", "3"),     / 1
            // avg = (2*3+1.75*4+3)/9 = 1.77777777778 ~ 1.7777777777777777
            // other weeks just have date shifted to Mondays
            vec![
                ("2022-11-07", "1.7777777777777777"),
                ("2022-11-28", "4"),
                ("2022-12-26", "0"),
                ("2023-01-30", "1"),
                ("2023-02-27", "2"),
            ],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_average_block_rewards_monthly() {
        simple_test_chart::<AverageBlockRewardsMonthly>(
            "update_average_block_rewards_monthly",
            vec![
                ("2022-11-01", "1.7777777777777777"),
                ("2022-12-01", "4"),
                ("2023-01-01", "0"),
                ("2023-02-01", "1"),
                ("2023-03-01", "2"),
            ],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_average_block_rewards_yearly() {
        simple_test_chart::<AverageBlockRewardsYearly>(
            "update_average_block_rewards_yearly",
            vec![
                // (2*3+1.75*4+3+4)/10 = 2
                ("2022-01-01", "2"),
                // (0+1+2)/3 = 1
                ("2023-01-01", "1"),
            ],
        )
        .await;
    }
}
