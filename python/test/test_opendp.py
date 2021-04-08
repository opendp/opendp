import opendp


def test_init():
    odp = opendp.OpenDP()
    assert odp

def test_data_object_int():
    odp = opendp.OpenDP()
    val_in = 123
    obj = odp.py_to_obj(val_in)
    val_out = odp.obj_to_py(obj)
    assert val_out == val_in

def test_data_object_float():
    odp = opendp.OpenDP()
    val_in = 123.123
    obj = odp.py_to_obj(val_in)
    val_out = odp.obj_to_py(obj)
    assert val_out == val_in

def test_data_object_str():
    odp = opendp.OpenDP()
    val_in = "hello, world"
    obj = odp.py_to_obj(val_in)
    val_out = odp.obj_to_py(obj)
    assert val_out == val_in

def test_data_object_list():
    odp = opendp.OpenDP()
    val_in = [1, 2, 3]
    obj = odp.py_to_obj(val_in)
    val_out = odp.obj_to_py(obj)
    assert val_out == val_in

def test_identity_int():
    odp = opendp.OpenDP()
    transformation = odp.trans.make_identity(b"<i32>")
    arg = 123
    ret = odp.transformation_invoke(transformation, arg)
    assert ret == arg

def test_identity_float():
    odp = opendp.OpenDP()
    transformation = odp.trans.make_identity(b"<f64>")
    arg = 123.123
    ret = odp.transformation_invoke(transformation, arg)
    assert ret == arg

def test_identity_str():
    odp = opendp.OpenDP()
    transformation = odp.trans.make_identity(b"<String>")
    arg = "hello, world"
    ret = odp.transformation_invoke(transformation, arg)
    assert ret == arg

def test_identity_list():
    odp = opendp.OpenDP()
    transformation = odp.trans.make_identity(b"<Vec<i32>>")
    arg = [1, 2, 3]
    ret = odp.transformation_invoke(transformation, arg)
    assert ret == arg
