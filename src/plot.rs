use egui_plotter::EguiBackend;
use plotters::{
    chart::ChartBuilder,
    prelude::{IntoDrawingArea, PathElement},
    series::LineSeries,
    style::{Color, IntoFont, BLACK, RED, WHITE},
};
use rustpython_ndarray::pyndarray::{PyNdArrayFloat32, PyNdArrayFloat64};

#[rustpython_vm::pymodule]
pub mod pyplotter {
    use super::*;
    use std::borrow::BorrowMut;
    use std::cell::{LazyCell, RefCell};

    use rustpython_vm::builtins::PyStr;
    use rustpython_vm::function::KwArgs;
    use rustpython_vm::{PyObjectRef, PyResult, VirtualMachine};

    thread_local! {
        static COMMANDS: LazyCell<RefCell<Vec<PlotCommand>>> = LazyCell::new(RefCell::default);
    }

    #[pyfunction]
    fn plot(
        x: PyObjectRef,
        y: PyObjectRef,
        mut kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<()> {
        let label: String = kwargs
            .pop_kwarg("label")
            .and_then(|label| label.downcast::<PyStr>().ok())
            .map(|py| py.to_string())
            .unwrap_or_default();
        let x = x
            .downcast::<PyNdArrayFloat32>()
            .map_err(|_| vm.new_runtime_error("X Must be float32".into()))?;
        let y = y
            .downcast::<PyNdArrayFloat32>()
            .map_err(|_| vm.new_runtime_error("Y Must be float32".into()))?;
        COMMANDS.with(|reader| {
            (**reader).borrow_mut().push(PlotCommand::PlotXY {
                x: (*x).clone(),
                y: (*y).clone(),
                label,
            })
        });
        Ok(())
    }

    #[pyfunction]
    fn title(title: String, vm: &VirtualMachine) -> PyResult<()> {
        COMMANDS.with(|reader| {
            (**reader)
                .borrow_mut()
                .push(PlotCommand::Title(title))
        });

        Ok(())
    }


    #[pyfunction]
    fn xlim(left: f32, right: f32, vm: &VirtualMachine) -> PyResult<()> {
        COMMANDS.with(|reader| {
            (**reader)
                .borrow_mut()
                .push(PlotCommand::Xlim { left, right })
        });

        Ok(())
    }

    #[pyfunction]
    fn ylim(bottom: f32, top: f32, vm: &VirtualMachine) -> PyResult<()> {
        COMMANDS.with(|reader| {
            (**reader)
                .borrow_mut()
                .push(PlotCommand::Ylim { bottom, top })
        });

        Ok(())
    }

    pub fn dump_commands() -> Vec<PlotCommand> {
        COMMANDS.with(|r| std::mem::take(&mut *(**r).borrow_mut()))
    }
}

pub enum PlotCommand {
    Title(String),
    PlotXY {
        x: PyNdArrayFloat32,
        y: PyNdArrayFloat32,
        label: String,
    },
    Xlim {
        left: f32,
        right: f32,
    },
    Ylim {
        bottom: f32,
        top: f32,
    },
}

pub fn draw_plots(ui: &egui::Ui, commands: &[PlotCommand]) -> Result<(), String> {
    let root = EguiBackend::new(&*ui).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut plot_left: f32 = -1.0;
    let mut plot_right: f32 = 1.0;
    let mut plot_top: f32 = 1.0;
    let mut plot_bottom: f32 = -1.0;
    let mut plot_title = String::new();

    for command in commands {
        match &command {
            PlotCommand::Title(title) => plot_title = title.clone(),
            PlotCommand::Ylim { bottom, top } => {
                plot_bottom = *bottom;
                plot_top = *top;
            }
            PlotCommand::Xlim { left, right } => {
                plot_left = *left;
                plot_right = *right;
            }
            PlotCommand::PlotXY { x, y, label } => {
                let mut chart = ChartBuilder::on(&root)
                    .caption(&plot_title, ("sans-serif", 25).into_font())
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(plot_left..plot_right, plot_bottom..plot_top)
                    .unwrap();

                chart.configure_mesh().draw().unwrap();

                if x.arr.read(|x| x.ndim() != 1) {
                    return Err("X must be 1 dimensional array".to_string());
                }

                if y.arr.read(|y| y.ndim() != 1) {
                    return Err("Y must be 1 dimensional array".to_string());
                }

                let coords = x.arr.read(|x| {
                    y.arr.read(|y| {
                        x.iter()
                            .copied()
                            .zip(y.iter().copied())
                            .collect::<Vec<(f32, f32)>>()
                    })
                });
                chart
                    .draw_series(LineSeries::new(coords, &RED))
                    .unwrap()
                    .label(label);
                    //.legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
                //legend = false;

                chart
                    .configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .unwrap();
            }
        }
    }

    root.present().unwrap();
    Ok(())
}
