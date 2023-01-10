from pprint import pprint

import gax
from gax import gam

data = gam.parse("data/example.gam")

data[0].annotation["name"] = "new name"

print(data[0].__repr__())

gam.write([data[0]], "data/example.out.gam")
