use opendp::dom::AllDomain;
use opendp::sarus::{make_pld_composition, make_pld_gaussian, make_pld_laplace};

use serde::{Deserialize, Serialize};
use vega_lite_4::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenDP example");
    // gaussian_plot_example()?;
    laplace_plot_example()?;
    laplace_comp_plot_example()?;

    // gaussian_comp_example()?;
    return Ok(());
}

fn gaussian_plot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Plot Gaussian example");
    let gauss_meas = make_pld_gaussian::<AllDomain<f64>>(1.0)?;
    plot_fs(vec![
        gauss_meas.output_measure.f(&1.0),
        gauss_meas.output_measure.f(&2.0),
    ])?;
    // let values: Vec<Point> = meas_0.output_measure.f(&1.0).into_iter().map(|(x,y)| Point {x,y}).collect();
    // the chart
    Ok(())
}

fn laplace_plot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Plot Laplace examples");
    let lap_meas = make_pld_laplace::<AllDomain<f64>>(0.1)?;
    plot_fs(vec![
        lap_meas.output_measure.f(&0.1),
        lap_meas.output_measure.f(&1.0),
    ])?;
    // let values: Vec<Point> = meas_0.output_measure.f(&1.0).into_iter().map(|(x,y)| Point {x,y}).collect();
    // the chart
    Ok(())
}

fn laplace_comp_plot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Laplace and Gauussian example");
    let meas_0 = make_pld_laplace::<AllDomain<f64>>(1.0)?;
    let meas_1 = make_pld_laplace::<AllDomain<f64>>(1.0)?;
    let comp_meas = make_pld_composition(&meas_0, &meas_1)?;
    plot_fs(vec![
        comp_meas.output_measure.f(&0.5),
        meas_0.output_measure.f(&1.0),
    ])?;
    Ok(())
}

fn plot_fs(fs: Vec<Vec<(f64,f64)>>) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Serialize, Deserialize)]

    pub struct Point {
        pub x: f64,
        pub y: f64,
        pub i: usize,
    }

    let values: Vec<Point> = fs.into_iter().enumerate().flat_map(|(i,f)| 
        f.into_iter().map(move |(x,y)| Point {x,y,i})
    ).collect();

    let chart = VegaliteBuilder::default()
        .title("Tradeoff function")
        .description("Tradeoff function.")
        .width(800.)
        .height(800.)
        .data(&values)
        .mark(DefBuilder::default()
            .def_type(Mark::Line)
            .point(true)
            .build()?
        )
        .encoding(
            EdEncodingBuilder::default()
            .x(XClassBuilder::default()
                .field("x")
                .position_def_type(Type::Quantitative)
                .build()?)
            .y(YClassBuilder::default()
                .field("y")
                .position_def_type(Type::Quantitative)
                .build()?)
            .color(ColorClassBuilder::default().field("i").build()?)
            .build()?
        )
        .build()?;

    // display the chart using `showata`
    chart.show()?;
    // // print the vega lite spec
    // eprint!("{}", chart.to_string()?);
    Ok(())
}