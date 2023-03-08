from gax import convert_gam_to_gaf, gam, load_graph

data = gam.parse("data/convert.gam")

print(data[0])

graph = load_graph("data/convert.gfa")
data = convert_gam_to_gaf(data, graph)
print(data[0])
