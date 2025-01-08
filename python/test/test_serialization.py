import json
import importlib

import opendp.prelude as dp


class Encoder(json.JSONEncoder):
    def default(self, obj):
        if hasattr(obj, 'log'):
            return self.default(obj.log)
        if isinstance(obj, dict):
            return {
                k: self.default(v)
                for k, v in obj.items()
            }
        if isinstance(obj, tuple):
            return {'__tuple': obj}
        if isinstance(obj, (str, int, float, bool, type(None))):
            return obj
        raise Exception(f'OpenDP JSON Encoder does not handle {obj}')
    
def deserialization_hook(dp_dict):
    if 'func' in dp_dict:
        module = importlib.import_module(f"opendp.{dp_dict['module']}")
        func = getattr(module, dp_dict["func"])
        return func(**dp_dict.get("kwargs", {}))
    if '__tuple' in dp_dict:
        return tuple(dp_dict['__tuple'])
    return dp_dict

def test_atom_domain_serialization():
    domain = dp.atom_domain(bounds=(0,10))
    serialized = json.dumps(domain, cls=Encoder)
    print('serialized:', serialized)
    deserialized = json.loads(serialized, object_hook=deserialization_hook)
    assert domain == deserialized