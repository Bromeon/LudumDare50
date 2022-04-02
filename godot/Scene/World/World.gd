extends Spatial

const GodotApi = preload("res://Native/GodotApi.gdns")

var api

func _ready():
	print("World is ready.")

	api = GodotApi.new()
	api.initialize("string from GDScript")
	add_child(api)


func _process(_delta):
	pass
