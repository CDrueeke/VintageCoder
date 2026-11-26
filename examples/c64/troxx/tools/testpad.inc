; =====================================================================================================================
; testpad.inc - Huellen fuer sehr kurze Testroutinen (nur mit -D TESTPAD=1; baut tools/build_pad.sh nach build/pad.prg)
; =====================================================================================================================
; ZWECK     Der Testlaeufer ruft eine Routine ueber eine Falle auf und wartet auf deren Rueckkehr.  Kehrt sie schneller zurueck, als der Emulator das
;           Fortsetzen bestaetigt, meldet der Laeufer "Fortsetzen nicht bestaetigt", obwohl nichts falsch ist.  Jede Huelle ruft die Routine auf und
;           wartet danach rund 15000 Takte, ohne Register und Flags zu veraendern.  Das Spiel selbst enthaelt keine Huellen.
; =====================================================================================================================

.macro pad(target)
        jsr target
        php
        pha
        txa
        pha
        tya
        pha
        ldx #0
        ldy #12
@w:     dex
        bne @w
        dey
        bne @w
        pla
        tay
        pla
        tax
        pla
        plp
        rts
.endmacro

pad_cell_type:    pad grid.cell_type
pad_nb_boxes:     pad grid.nb_boxes
pad_rnd:          pad enemies.rnd
pad_pick_random:  pad enemies.pick_random
pad_level_idx:    pad enemies.level_idx
pad_set_tempo:    pad set_tempo
pad_read_port:    pad input.read_port
pad_port_of:      pad input.port_of
pad_title_change: pad title_change
pad_items_check:  pad items.check
pad_sound_play:   pad sound.play
pad_switch_player: pad switch_player
pad_box_add:      pad grid.box_add
pad_seg_is_closed: pad grid.seg_is_closed
pad_bcd_sub:      pad hud.bcd_sub
pad_place:        pad hiscore.place
