extends MeshInstance


func _ready():
	owner.connect("note_play", self, "_note_played")
	owner.connect("note_ended", self, "_note_ended")


func _note_played(note):
	get_node("Key%s" % note).press()


func _note_ended(note):
	get_node("Key%s" % note).unpress()
