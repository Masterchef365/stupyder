use rustpython_ndarray::pyndarray::{PyNdArrayFloat32, PyNdArrayFloat64};

#[rustpython_vm::pymodule]
pub mod pyplotter {
    use super::*;
    use std::cell::{LazyCell, RefCell};
    use std::borrow::BorrowMut;

    use rustpython_vm::{PyObjectRef, PyResult, VirtualMachine};

    thread_local! {
        static COMMANDS: LazyCell<RefCell<Vec<PlotCommand>>> = LazyCell::new(RefCell::default);
    }

    #[pyfunction]
    fn plot(x: PyObjectRef, y: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
        let x = x.downcast::<PyNdArrayFloat32>().map_err(|_| vm.new_runtime_error("X Must be float32".into()))?;
        let y = y.downcast::<PyNdArrayFloat32>().map_err(|_| vm.new_runtime_error("Y Must be float32".into()))?;
        COMMANDS.with(|reader| (**reader).borrow_mut().push(PlotCommand::PlotXY { 
            x: (*x).clone(),
            y: (*y).clone(),
        }));
        Ok(())
    }

    pub fn dump_commands() -> Vec<PlotCommand> {
        COMMANDS.with(|r| std::mem::take(&mut *(**r).borrow_mut()))
    }
}

pub enum PlotCommand {
    PlotXY {
        x: PyNdArrayFloat32, 
        y: PyNdArrayFloat32,
    },
}
