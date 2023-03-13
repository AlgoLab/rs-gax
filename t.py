from gax import convert_gaf_to_gam, convert_gam_to_gamp, gaf, load_graph

data = gaf.parse("data/convert.gaf")
graph = load_graph("data/convert.gfa")


print(data[0])
data = convert_gaf_to_gam(data, graph)
print(data[0])
data = convert_gam_to_gamp(data)
print(data[0])
