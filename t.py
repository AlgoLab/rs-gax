from gax import gam

data = gam.parse("data/example.gam")

print(data[0])

gam.write(data, "data/example.out.gam")

