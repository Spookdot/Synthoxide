extends Node

signal note_played(key_num)
signal note_stopped(key_num)

func _input(event):
	if event is InputEventMIDI:
		if event.message == MIDI_MESSAGE_NOTE_ON:
			emit_signal("note_played", event.pitch)
		if event.message == MIDI_MESSAGE_NOTE_OFF:
			emit_signal("note_stopped", event.pitch)
	if event is InputEventKey:
		emit_signal("note_played", 45)
