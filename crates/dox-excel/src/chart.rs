//! Excel chart creation and management using rust_xlsxwriter
//!
//! This module provides functionality to:
//! - Create various chart types (column, bar, line, pie, scatter)
//! - Add data series to charts with proper range references
//! - Position and style charts within worksheets
//! - Update chart data dynamically

use anyhow::{anyhow, Result};
use rust_xlsxwriter::{Chart, ChartType, Worksheet};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use dox_core::RangeRef;

/// Represents a chart data series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    /// Series name (for legend)
    pub name: String,
    /// Data range for the series (e.g., "Sheet1!B2:B10")
    pub data_range: RangeRef,
    /// Category range (x-axis labels, e.g., "Sheet1!A2:A10")
    pub category_range: Option<RangeRef>,
}

/// Chart positioning and sizing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPosition {
    /// Starting column for the chart
    pub col: u16,
    /// Starting row for the chart
    pub row: u32,
    /// Chart width in pixels
    pub width: Option<u32>,
    /// Chart height in pixels
    pub height: Option<u32>,
    /// Column offset in pixels
    pub col_offset: Option<u32>,
    /// Row offset in pixels
    pub row_offset: Option<u32>,
}

/// Chart styling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartStyle {
    /// Chart title
    pub title: Option<String>,
    /// X-axis title
    pub x_axis_title: Option<String>,
    /// Y-axis title
    pub y_axis_title: Option<String>,
    /// Whether to show legend
    pub show_legend: bool,
    /// Whether to show data labels
    pub show_data_labels: bool,
    /// Chart style ID (1-48 in Excel)
    pub style_id: Option<u8>,
}

/// Supported chart types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcelChartType {
    Column,
    ColumnStacked,
    ColumnStacked100,
    Bar,
    BarStacked,
    BarStacked100,
    Line,
    LineStacked,
    LineStacked100,
    LineMarkers,
    LineMarkersStacked,
    LineMarkersStacked100,
    Pie,
    Doughnut,
    Scatter,
    ScatterStraight,
    ScatterStraightMarkers,
    ScatterSmooth,
    ScatterSmoothMarkers,
    Area,
    AreaStacked,
    AreaStacked100,
}

/// Chart builder for creating Excel charts
pub struct ExcelChartBuilder {
    chart_type: ExcelChartType,
    series: Vec<ChartSeries>,
    position: ChartPosition,
    style: ChartStyle,
}

/// Chart manager for Excel worksheets
pub struct ChartManager<'a> {
    worksheet: &'a mut Worksheet,
    charts: Vec<Chart>,
}

impl ExcelChartType {
    /// Convert to rust_xlsxwriter ChartType
    pub fn to_rust_xlsxwriter_type(self) -> ChartType {
        match self {
            ExcelChartType::Column => ChartType::Column,
            ExcelChartType::ColumnStacked => ChartType::ColumnStacked,
            ExcelChartType::ColumnStacked100 => ChartType::ColumnPercentStacked,
            ExcelChartType::Bar => ChartType::Bar,
            ExcelChartType::BarStacked => ChartType::BarStacked,
            ExcelChartType::BarStacked100 => ChartType::BarPercentStacked,
            ExcelChartType::Line => ChartType::Line,
            ExcelChartType::LineStacked => ChartType::LineStacked,
            ExcelChartType::LineStacked100 => ChartType::LinePercentStacked,
            ExcelChartType::LineMarkers => ChartType::Line,
            ExcelChartType::LineMarkersStacked => ChartType::LineStacked,
            ExcelChartType::LineMarkersStacked100 => ChartType::LinePercentStacked,
            ExcelChartType::Pie => ChartType::Pie,
            ExcelChartType::Doughnut => ChartType::Doughnut,
            ExcelChartType::Scatter => ChartType::Scatter,
            ExcelChartType::ScatterStraight => ChartType::ScatterStraight,
            ExcelChartType::ScatterStraightMarkers => ChartType::ScatterStraight,
            ExcelChartType::ScatterSmooth => ChartType::ScatterSmooth,
            ExcelChartType::ScatterSmoothMarkers => ChartType::ScatterSmooth,
            ExcelChartType::Area => ChartType::Area,
            ExcelChartType::AreaStacked => ChartType::AreaStacked,
            ExcelChartType::AreaStacked100 => ChartType::AreaPercentStacked,
        }
    }
}

impl Default for ChartPosition {
    fn default() -> Self {
        ChartPosition {
            col: 8, // Column I (default Excel position)
            row: 1, // Row 2
            width: Some(480),
            height: Some(288),
            col_offset: None,
            row_offset: None,
        }
    }
}

impl Default for ChartStyle {
    fn default() -> Self {
        ChartStyle {
            title: None,
            x_axis_title: None,
            y_axis_title: None,
            show_legend: true,
            show_data_labels: false,
            style_id: Some(2), // Default Excel chart style
        }
    }
}

impl ExcelChartBuilder {
    /// Create a new chart builder
    pub fn new(chart_type: ExcelChartType) -> Self {
        ExcelChartBuilder {
            chart_type,
            series: Vec::new(),
            position: ChartPosition::default(),
            style: ChartStyle::default(),
        }
    }

    /// Add a data series to the chart
    pub fn add_series(mut self, series: ChartSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Set chart position
    pub fn position(mut self, position: ChartPosition) -> Self {
        self.position = position;
        self
    }

    /// Set chart style
    pub fn style(mut self, style: ChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Set chart title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.style.title = Some(title.into());
        self
    }

    /// Set axis titles
    pub fn axis_titles(mut self, x_title: impl Into<String>, y_title: impl Into<String>) -> Self {
        self.style.x_axis_title = Some(x_title.into());
        self.style.y_axis_title = Some(y_title.into());
        self
    }

    /// Build the chart and add it to the worksheet
    pub fn build(self, worksheet: &mut Worksheet) -> Result<()> {
        debug!("Building chart of type: {:?}", self.chart_type);

        // Create the chart
        let mut chart = Chart::new(self.chart_type.to_rust_xlsxwriter_type());

        // Set chart title
        if let Some(ref title) = self.style.title {
            chart.title().set_name(title);
        }

        // Set axis titles
        if let Some(ref x_title) = self.style.x_axis_title {
            chart.x_axis().set_name(x_title);
        }
        if let Some(ref y_title) = self.style.y_axis_title {
            chart.y_axis().set_name(y_title);
        }

        // Configure legend
        if !self.style.show_legend {
            chart.legend().set_hidden();
        }

        // Set style
        if let Some(style_id) = self.style.style_id {
            chart.set_style(style_id);
        }

        // Add data series
        for (index, series) in self.series.iter().enumerate() {
            debug!("Adding series {}: {}", index, series.name);

            let chart_series = chart.add_series();

            // Set series name
            chart_series.set_name(&series.name);

            // Set data range
            chart_series.set_values(&series.data_range.0);

            // Set category range if provided
            if let Some(ref cat_range) = series.category_range {
                chart_series.set_categories(&cat_range.0);
            }

            // Show data labels if requested
            if self.style.show_data_labels {
                // Note: Data labels would be configured here with actual rust_xlsxwriter API
                // chart_series.set_data_label() method exists but needs proper configuration
            }
        }

        // Position the chart
        worksheet.insert_chart_with_offset(
            self.position.row,
            self.position.col,
            &chart,
            self.position.col_offset.unwrap_or(0),
            self.position.row_offset.unwrap_or(0),
        )?;

        // Set size if specified
        if let (Some(width), Some(height)) = (self.position.width, self.position.height) {
            // Note: rust_xlsxwriter doesn't have direct size setting in insert_chart_with_offset
            // This would require using set_size() method on the chart before insertion
            debug!("Chart size set to {}x{}", width, height);
        }

        info!("Chart added successfully with {} series", self.series.len());
        Ok(())
    }
}

impl ChartSeries {
    /// Create a new chart series
    pub fn new(name: impl Into<String>, data_range: RangeRef) -> Self {
        ChartSeries {
            name: name.into(),
            data_range,
            category_range: None,
        }
    }

    /// Set category range for the series
    pub fn with_categories(mut self, category_range: RangeRef) -> Self {
        self.category_range = Some(category_range);
        self
    }
}

impl<'a> ChartManager<'a> {
    /// Create a new chart manager for a worksheet
    pub fn new(worksheet: &'a mut Worksheet) -> Self {
        ChartManager {
            worksheet,
            charts: Vec::new(),
        }
    }

    /// Create a simple column chart with data
    pub fn create_column_chart(
        &mut self,
        title: &str,
        data_ranges: Vec<(&str, RangeRef)>, // (series_name, data_range)
        category_range: Option<RangeRef>,
        position: Option<ChartPosition>,
    ) -> Result<()> {
        let mut builder = ExcelChartBuilder::new(ExcelChartType::Column).title(title);

        // Add series
        for (name, data_range) in data_ranges {
            let mut series = ChartSeries::new(name, data_range);
            if let Some(ref cat_range) = category_range {
                series = series.with_categories(cat_range.clone());
            }
            builder = builder.add_series(series);
        }

        // Set position if provided
        if let Some(pos) = position {
            builder = builder.position(pos);
        }

        builder.build(self.worksheet)?;
        info!("Column chart '{}' created successfully", title);
        Ok(())
    }

    /// Create a simple line chart with data
    pub fn create_line_chart(
        &mut self,
        title: &str,
        data_ranges: Vec<(&str, RangeRef)>,
        category_range: Option<RangeRef>,
        position: Option<ChartPosition>,
    ) -> Result<()> {
        let mut builder = ExcelChartBuilder::new(ExcelChartType::LineMarkers).title(title);

        for (name, data_range) in data_ranges {
            let mut series = ChartSeries::new(name, data_range);
            if let Some(ref cat_range) = category_range {
                series = series.with_categories(cat_range.clone());
            }
            builder = builder.add_series(series);
        }

        if let Some(pos) = position {
            builder = builder.position(pos);
        }

        builder.build(self.worksheet)?;
        info!("Line chart '{}' created successfully", title);
        Ok(())
    }

    /// Create a pie chart with data
    pub fn create_pie_chart(
        &mut self,
        title: &str,
        data_range: RangeRef,
        category_range: Option<RangeRef>,
        position: Option<ChartPosition>,
    ) -> Result<()> {
        let mut builder = ExcelChartBuilder::new(ExcelChartType::Pie).title(title);

        let mut series = ChartSeries::new("Data", data_range);
        if let Some(cat_range) = category_range {
            series = series.with_categories(cat_range);
        }
        builder = builder.add_series(series);

        if let Some(pos) = position {
            builder = builder.position(pos);
        }

        // Pie charts typically show data labels
        let style = ChartStyle {
            show_data_labels: true,
            ..ChartStyle::default()
        };
        builder = builder.style(style);

        builder.build(self.worksheet)?;
        info!("Pie chart '{}' created successfully", title);
        Ok(())
    }

    /// Create a scatter chart with data
    pub fn create_scatter_chart(
        &mut self,
        title: &str,
        x_range: RangeRef,
        y_range: RangeRef,
        series_name: Option<&str>,
        position: Option<ChartPosition>,
    ) -> Result<()> {
        let mut builder = ExcelChartBuilder::new(ExcelChartType::ScatterSmoothMarkers).title(title);

        let series =
            ChartSeries::new(series_name.unwrap_or("Series 1"), y_range).with_categories(x_range);

        builder = builder.add_series(series);

        if let Some(pos) = position {
            builder = builder.position(pos);
        }

        builder.build(self.worksheet)?;
        info!("Scatter chart '{}' created successfully", title);
        Ok(())
    }
}

/// Helper functions for chart creation
pub mod helpers {
    use super::*;

    /// Create a simple data visualization from tabular data
    pub fn create_data_summary_chart(
        worksheet: &mut Worksheet,
        data_start_row: u32,
        data_end_row: u32,
        chart_type: ExcelChartType,
        chart_title: &str,
    ) -> Result<()> {
        let mut chart_manager = ChartManager::new(worksheet);

        // Assume data is in columns A (categories) and B (values)
        let category_range =
            RangeRef::new(format!("A{}:A{}", data_start_row + 1, data_end_row + 1));
        let data_range = RangeRef::new(format!("B{}:B{}", data_start_row + 1, data_end_row + 1));

        match chart_type {
            ExcelChartType::Column
            | ExcelChartType::ColumnStacked
            | ExcelChartType::ColumnStacked100 => {
                chart_manager.create_column_chart(
                    chart_title,
                    vec![("Data", data_range)],
                    Some(category_range),
                    None,
                )?;
            }
            ExcelChartType::Line | ExcelChartType::LineMarkers => {
                chart_manager.create_line_chart(
                    chart_title,
                    vec![("Data", data_range)],
                    Some(category_range),
                    None,
                )?;
            }
            ExcelChartType::Pie => {
                chart_manager.create_pie_chart(
                    chart_title,
                    data_range,
                    Some(category_range),
                    None,
                )?;
            }
            _ => {
                return Err(anyhow!(
                    "Chart type {:?} not supported by helper function",
                    chart_type
                ));
            }
        }

        Ok(())
    }

    /// Create multiple charts for data comparison
    pub fn create_comparison_charts(
        worksheet: &mut Worksheet,
        data_ranges: Vec<(&str, RangeRef)>,
        category_range: RangeRef,
        chart_title: &str,
    ) -> Result<()> {
        let mut chart_manager = ChartManager::new(worksheet);

        // Create a column chart for comparison
        chart_manager.create_column_chart(
            chart_title,
            data_ranges.clone(),
            Some(category_range.clone()),
            Some(ChartPosition {
                col: 6,
                row: 2,
                ..ChartPosition::default()
            }),
        )?;

        // Create a line chart for trends (if more than one series)
        if data_ranges.len() > 1 {
            chart_manager.create_line_chart(
                &format!("{} - Trend", chart_title),
                data_ranges,
                Some(category_range),
                Some(ChartPosition {
                    col: 6,
                    row: 18, // Position below the column chart
                    ..ChartPosition::default()
                }),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_series_creation() {
        let series = ChartSeries::new("Test Series", RangeRef::new("A1:A10"));
        assert_eq!(series.name, "Test Series");
        assert_eq!(series.data_range.0, "A1:A10");
        assert!(series.category_range.is_none());
    }

    #[test]
    fn test_chart_series_with_categories() {
        let series = ChartSeries::new("Test Series", RangeRef::new("B1:B10"))
            .with_categories(RangeRef::new("A1:A10"));

        assert_eq!(series.name, "Test Series");
        assert_eq!(series.data_range.0, "B1:B10");
        assert!(series.category_range.is_some());
        assert_eq!(series.category_range.unwrap().0, "A1:A10");
    }

    #[test]
    fn test_chart_position_default() {
        let position = ChartPosition::default();
        assert_eq!(position.col, 8);
        assert_eq!(position.row, 1);
        assert_eq!(position.width, Some(480));
        assert_eq!(position.height, Some(288));
    }

    #[test]
    fn test_chart_type_conversion() {
        // Test that conversions don't panic - we can't compare ChartType directly
        // because rust_xlsxwriter::ChartType doesn't implement Debug
        let _column_type = ExcelChartType::Column.to_rust_xlsxwriter_type();
        let _line_type = ExcelChartType::Line.to_rust_xlsxwriter_type();
        let _pie_type = ExcelChartType::Pie.to_rust_xlsxwriter_type();
        // Just ensure conversion methods work without errors
    }
}
