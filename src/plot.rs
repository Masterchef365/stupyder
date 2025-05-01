use rustpython_ndarray::pyndarray::PyNdArrayFloat64;

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
        let x = x.downcast::<PyNdArrayFloat64>().map_err(|_| vm.new_runtime_error("X Must be float32".into()))?;
        let y = y.downcast::<PyNdArrayFloat64>().map_err(|_| vm.new_runtime_error("Y Must be float32".into()))?;
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
        x: PyNdArrayFloat64, 
        y: PyNdArrayFloat64,
    },
}

/*
   #[derive(PyPayload, Clone, Debug)]
   #[pyclass(module = "pyndarray", name)]
   pub struct $dtype {
   pub(crate) arr: SlicedArcArray<$primitive>,
   }

   impl GenericArray for SlicedArcArray<$primitive> {
   type PyArray = $dtype;
   const DTYPE: DataType = $dtype_enum;
   fn cast(&self) -> Self::PyArray {
   $dtype { arr: self.clone() }
   }
   }

   #[pyclass]
   impl $dtype {
// AsMapping methods
#[pymethod(magic)]
fn getitem(&self, needle: PyObjectRef, vm: &VirtualMachine) -> PyResult {
self.arr.getitem(needle, vm)
}

#[pymethod(magic)]428016b3948d04c4395929a89a89172ba5b8364e
fn setitem(
&self,
needle: PyObjectRef,
value: PyObjectRef,
vm: &VirtualMachine,
) -> PyResult<()> {
let slice = py_index_to_sliceinfo(needle, vm)?;
self.assign_or_elem_fn(
slice,
value,
vm,
|mut dest, src, _| Ok(dest.assign(&src)),
|mut dest, value, _| Ok(dest.fill(value)),
)
}

#[pymethod(magic)]
fn len(
&self,
_vm: &VirtualMachine,
) -> PyResult<PyInt> {
let len = self.arr.read(|sliced| sliced.len());
Ok(len.into())
}

// Stringy methods
#[pymethod(magic)]
fn str(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult<PyStrRef> {
Ok(vm.ctx.new_str(zelf.arr.to_string()))
}

#[pymethod(magic)]
fn repr(zelf: PyRef<Self>, vm: &VirtualMachine) -> PyResult<PyStrRef> {
Ok(vm.ctx.new_str(zelf.arr.repr()))
}

// AsNumber methods
#[pymethod(magic)]
fn iadd(
zelf: PyRef<Self>,
other: PyObjectRef,
vm: &VirtualMachine,
) -> PyResult<()> {
let empty_slice = empty_slice_like(&zelf.arr);
zelf.assign_or_elem_fn(
empty_slice,
other,
vm,
    |mut dest, src, _vm| Ok(dest += &src),
    |mut dest, value, _vm| Ok(dest += value),
    )
    }

#[pymethod(magic)]
fn add(zelf: PyRef<Self>, other: PyObjectRef, vm: &VirtualMachine) -> PyResult {
    let inst = $dtype {
        arr: zelf.arr.sliced_copy(),
    };
    let inst = inst.into_ref(&vm.ctx);
    $dtype::iadd(inst.clone(), other, vm)?;
    Ok(inst.into())
}

#[pymethod(magic)]
fn isub(
    zelf: PyRef<Self>,
    other: PyObjectRef,
    vm: &VirtualMachine,
) -> PyResult<()> {
    let empty_slice = empty_slice_like(&zelf.arr);
    zelf.assign_or_elem_fn(
        empty_slice,
        other,
        vm,
        |mut dest, src, _vm| Ok(dest -= &src),
        |mut dest, value, _vm| Ok(dest -= value),
    )
}

#[pymethod(magic)]
fn sub(zelf: PyRef<Self>, other: PyObjectRef, vm: &VirtualMachine) -> PyResult {
    let inst = $dtype {
        arr: zelf.arr.sliced_copy(),
    };
    let inst = inst.into_ref(&vm.ctx);
    $dtype::isub(inst.clone(), other, vm)?;
    Ok(inst.into())
}

#[pymethod(magic)]
fn itruediv(
    zelf: PyRef<Self>,
    other: PyObjectRef,
    vm: &VirtualMachine,
) -> PyResult<()> {
    let empty_slice = empty_slice_like(&zelf.arr);
    zelf.assign_or_elem_fn(
        empty_slice,
        other,
        vm,
        |mut dest, src, _vm| Ok(dest /= &src),
        |mut dest, value, _vm| Ok(dest /= value),
    )
}

#[pymethod(magic)]
fn truediv(zelf: PyRef<Self>, other: PyObjectRef, vm: &VirtualMachine) -> PyResult {
    let inst = $dtype {
        arr: zelf.arr.sliced_copy(),
    };
    let inst = inst.into_ref(&vm.ctx);
    $dtype::itruediv(inst.clone(), other, vm)?;
    Ok(inst.into())
}

#[pymethod(magic)]
fn imul(
    zelf: PyRef<Self>,
    other: PyObjectRef,
    vm: &VirtualMachine,
) -> PyResult<()> {
    let empty_slice = empty_slice_like(&zelf.arr);
    zelf.assign_or_elem_fn(
        empty_slice,
        other,
        vm,
        |mut dest, src, _vm| Ok(dest *= &src),
        |mut dest, value, _vm| Ok(dest *= value),
    )
}

#[pymethod(magic)]
fn mul(zelf: PyRef<Self>, other: PyObjectRef, vm: &VirtualMachine) -> PyResult {
    let inst = $dtype {
        arr: zelf.arr.sliced_copy(),
    };
    let inst = inst.into_ref(&vm.ctx);
    $dtype::imul(inst.clone(), other, vm)?;
    Ok(inst.into())
}
}

impl $dtype {
    pub fn assign_or_elem_fn<F, G, U>(
        &self,
        slice: DynamicSlice,
        value: PyObjectRef,
        vm: &VirtualMachine,
        assign_fn: F,
        elem_fn: G,
    ) -> PyResult<U>
        where
        F: Fn(
            ArrayViewMutD<'_, $primitive>,
            ArrayViewD<'_, $primitive>,
            &VirtualMachine,
        ) -> PyResult<U>,
    G: Fn(
        ArrayViewMutD<'_, $primitive>,
        $primitive,
        &VirtualMachine,
    ) -> PyResult<U>,
    {
        if let Some(other_array) = value.downcast_ref::<$dtype>() {
            self.arr
                .assign_fn(slice, other_array.arr.clone(), vm, assign_fn)
        } else {
            let value: $primitive = TryFromObject::try_from_object(vm, value)?;
            self.arr.write(|mut sliced| {
                if let Err(e) = sliced.bounds_check(&slice) {
                    return Err(
                        vm.new_runtime_error(format!("Slice out of bounds; {e}"))
                    );
                }

                elem_fn(sliced.slice_mut(&slice), value, vm)
            })
        }
    }
}

impl AsMapping for $dtype {
    fn as_mapping() -> &'static PyMappingMethods {
        static AS_MAPPING: PyMappingMethods = PyMappingMethods {
            subscript: atomic_func!(|mapping, needle, vm| {
                $dtype::mapping_downcast(mapping).getitem(needle.to_pyobject(vm), vm)
            }),
            ass_subscript: atomic_func!(|mapping, needle, value, vm| {
                let zelf = $dtype::mapping_downcast(mapping);
                if let Some(value) = value {
                    zelf.setitem(needle.to_pyobject(vm), value, vm)
                } else {
                    //zelf.internal_delitem(needle, vm)
                    Err(vm.new_exception_msg(
                            vm.ctx.exceptions.runtime_error.to_owned(),
                            "Arrays do not support delete".to_string(),
                    ))
                }
            }),
            length: atomic_func!(|mapping, _vm| {
                let zelf = $dtype::mapping_downcast(mapping);
                Ok(zelf.arr.length())
            }),
        };
        &AS_MAPPING
    }
}

impl AsNumber for $dtype {
    fn as_number() -> &'static rustpython_vm::protocol::PyNumberMethods {
        static AS_MAPPING: PyNumberMethods = PyNumberMethods {
            inplace_add: Some(|a, b, vm| {
                $dtype::iadd(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )?;
                Ok(a.to_owned())
            }),
            add: Some(|a, b, vm| {
                $dtype::add(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )
            }),

            inplace_multiply: Some(|a, b, vm| {
                $dtype::imul(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )?;
                Ok(a.to_owned())
            }),
            multiply: Some(|a, b, vm| {
                $dtype::mul(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )
            }),

            inplace_true_divide: Some(|a, b, vm| {
                $dtype::itruediv(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )?;
                Ok(a.to_owned())
            }),
            true_divide: Some(|a, b, vm| {
                $dtype::truediv(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )
            }),

            inplace_subtract: Some(|a, b, vm| {
                $dtype::isub(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )?;
                Ok(a.to_owned())
            }),
            subtract: Some(|a, b, vm| {
                $dtype::sub(
                    $dtype::number_downcast_exact(a.to_number(), vm),
                    b.to_owned(),
                    vm,
                )
            }),

            ..PyNumberMethods::NOT_IMPLEMENTED
        };
        &AS_MAPPING
    }
}

impl From<SlicedArcArray<$primitive>> for $dtype {
    fn from(arr: SlicedArcArray<$primitive>) -> Self {
        Self { arr }
    }
}
};
}

build_pyarray!(f32, PyNdArrayFloat32, DataType::Float32);
build_pyarray!(f64, PyNdArrayFloat64, DataType::Float64);

#[pyfunction]
fn zeros(shape: PyObjectRef, mut kw: KwArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
    let dtype = kw.pop_kwarg("dtype");

    let shape = py_shape_to_rust(shape.into(), vm)?;

    let dtype = dtype
        .map(|dtype| {
            DataType::from_pyobject(&dtype)
                .ok_or_else(|| vm.new_runtime_error(format!("Unrecognized dtype {dtype:?}")))
        })
    .transpose()?;

    match dtype {
        Some(DataType::Float64) => Ok(PyNdArrayFloat64::from(SlicedArcArray::from_array(
                    ndarray::ArrayD::zeros(shape),
        ))
            .to_pyobject(vm)),
        None | Some(DataType::Float32) => Ok(PyNdArrayFloat32::from(
                SlicedArcArray::from_array(ndarray::ArrayD::zeros(shape)),
        )
            .to_pyobject(vm)),
    }
}

#[pyfunction]
fn arange(
    start_or_stop_a: PyRef<PyFloat>,
    stop: OptionalArg<PyRef<PyFloat>>,
    step: OptionalArg<PyRef<PyFloat>>,
    mut kw: KwArgs,
    vm: &VirtualMachine,
) -> PyResult {
    let dtype = kw.pop_kwarg("dtype");
    let dtype = dtype
        .map(|dtype| {
            DataType::from_pyobject(&dtype)
                .ok_or_else(|| vm.new_runtime_error(format!("Unrecognized dtype {dtype:?}")))
        })
    .transpose()?;
    let dtype = dtype.unwrap_or(DataType::Float32);

    let start_or_stop_a = start_or_stop_a.to_f64(); //pyint_to_isize(&start_or_stop_a, vm)?;
    let stop = stop.as_option().map(|stop| stop.to_f64()); //pyint_to_isize(&stop, vm)).transpose()?;
    let step = step.as_option().map(|step| step.to_f64()); //pyint_to_isize(&step, vm)).transpose()?;

    let (start, stop, step) = match (stop, step) {
        (None, None) => (0.0, start_or_stop_a, 1.0),
        (Some(stop), None) => (start_or_stop_a, stop, 1.0),
        (Some(stop), Some(step)) => (start_or_stop_a, stop, step),
        _ => unreachable!(),
    };

    Ok(match dtype {
        DataType::Float32 => SlicedArcArray::from_array(
            ndarray::Array::range(start as f32, stop as f32, step as f32).into_dyn(),
        )
            .cast()
            .to_pyobject(vm),
        DataType::Float64 => {
            SlicedArcArray::from_array(ndarray::Array::range(start, stop, step).into_dyn())
                .cast()
                .to_pyobject(vm)
        }
    })
}
*/
