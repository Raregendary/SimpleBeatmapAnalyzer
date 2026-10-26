use std::path::{Path, PathBuf};

use simple_beatmap_analyzer::{
    analyze_bytes, analyze_file, analyze_paths, find_beatmaps, parse_columns, parse_metadata,
    write_csv, AnalyzeOptions, Cache, Column, FileAnalysis, Mode,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sba-test-{}-{}-{}",
        std::process::id(),
        name,
        unique
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn metadata_handles_colons_unicode_ids_and_section_order() {
    let text = "\u{feff}osu file format v14\n\n[Metadata]\nTitle:A: B:C\nTitleUnicode:Ünïcode\nArtist:Artist\nBeatmapID:42\nBeatmapSetID:43\nTags:one two\n\n[General]\nAudioFilename:audio.mp3\nMode:2\n";
    let metadata = parse_metadata(text);

    assert_eq!(metadata.format_version, 14);
    assert_eq!(metadata.title, "A: B:C");
    assert_eq!(metadata.title_unicode, "Ünïcode");
    assert_eq!(metadata.beatmap_id, 42);
    assert_eq!(metadata.beatmap_set_id, 43);
    assert_eq!(metadata.mode, Mode::Catch);
    assert_eq!(metadata.tags, vec!["one", "two"]);
    assert_eq!(metadata.audio_filename, "audio.mp3");

    let from_fixture =
        parse_metadata(&std::fs::read_to_string(fixture("standard_50.osu")).unwrap());
    assert_eq!(from_fixture.mode, Mode::Standard);
    assert_eq!(from_fixture.beatmap_set_id, 6789);
    assert_eq!(from_fixture.artist_unicode, "Тестовый Артист");
}

#[test]
fn standard_fixture_produces_expected_patterns() {
    let analysis = analyze_file(fixture("standard_50.osu"), &AnalyzeOptions::default())
        .expect("standard fixture should analyze");

    assert_eq!(analysis.md5.len(), 32);
    assert_eq!(analysis.metadata.title, "Fixture Song: The Rebirth");
    assert_eq!(analysis.metadata.version, "Insane");
    assert_eq!(analysis.timing.objects, 50);
    assert_eq!(analysis.timing.circles, 50);
    assert!((analysis.timing.bpm - 120.0).abs() < 0.01);
    assert!((analysis.timing.playable_length_ms - 10_500.0).abs() < 0.01);
    assert!(analysis.difficulty.stars_nm > 0.0);
    assert!(analysis.difficulty.stars_dt >= analysis.difficulty.stars_nm);
    assert!(analysis.difficulty.pp_nm > 0.0);
    assert_eq!(analysis.difficulty.max_combo, 50);

    let patterns = analysis.patterns.expect("standard map has patterns");
    assert_eq!(patterns.total_notes, 50);
    assert_eq!(patterns.doubles, 2);
    assert_eq!(patterns.triples, 3);
    assert_eq!(patterns.quads, 0);
    assert_eq!(patterns.burst_notes, 8);
    assert_eq!(patterns.stream_notes, 20);
    assert_eq!(patterns.deathstream_notes, 0);
    assert_eq!(patterns.longest_stream, 20);
    assert_eq!(patterns.short_jump_notes, 0);
    assert_eq!(patterns.mid_jump_notes, 21);
    assert_eq!(patterns.long_jump_notes, 0);
    assert_eq!(patterns.jump_notes(), 21);
    assert_eq!(patterns.jump_notes_under_8, 0);
    assert_eq!(patterns.jump_notes_8_plus, 21);
    assert_eq!(
        patterns.jump_notes(),
        patterns.jump_notes_under_8 + patterns.jump_notes_8_plus
    );

    assert!((patterns.stream_pct() - 40.0).abs() < 0.01);
    assert!((patterns.burst_pct() - 16.0).abs() < 0.01);
    assert!((patterns.doubles_pct() - 4.0).abs() < 0.01);
    assert!((patterns.triples_pct() - 6.0).abs() < 0.01);
    assert!((patterns.mid_jump_pct() - 42.0).abs() < 0.01);

    // Averages divide by the moves that were actually measured (20), not by
    // the inflated note count (21).
    assert!((patterns.avg_jump_distance - 283.18).abs() < 0.5);
    assert!((patterns.avg_jump_speed - 1132.74).abs() < 2.0);

    assert!((patterns.si - 0.11).abs() < 0.001);
    assert!((patterns.ji + 0.15).abs() < 0.001);
    assert!((patterns.fcdbi + 0.91).abs() < 0.001);
}

#[test]
fn options_change_the_pattern_detection() {
    let path = fixture("options_probe.osu");

    let defaults = analyze_file(&path, &AnalyzeOptions::default())
        .unwrap()
        .patterns
        .unwrap();
    assert_eq!(defaults.stream_notes, 0);
    assert_eq!(defaults.short_jump_notes, 8);
    assert_eq!(defaults.jump_notes_under_8, 0);
    assert_eq!(defaults.jump_notes_8_plus, 8);
    assert!((defaults.long_jump_pct_8_plus() - 33.333).abs() < 0.01);
    assert!((defaults.avg_jump_distance - 150.04).abs() < 0.5);

    let wide = AnalyzeOptions {
        stream_distance: 100.0,
        jump_distance: 300.0,
        accuracy: 99.0,
    };
    let wide = analyze_file(&path, &wide).unwrap().patterns.unwrap();
    assert_eq!(wide.stream_notes, 16);
    assert_eq!(wide.short_jump_notes, 0);
    assert_eq!(wide.jump_notes(), 0);
    assert!((wide.stream_pct() - 66.666).abs() < 0.01);
}

#[test]
fn gui_jump_split_is_reported() {
    use rosu_pp::beatmap::TimingPoint;
    use rosu_pp::parse::{HitObject, HitObjectKind, Pos2};

    let objects: Vec<HitObject> = [
        (100, 100, 1000),
        (400, 300, 1250),
        (100, 100, 1500),
        (400, 300, 1750),
    ]
    .into_iter()
    .map(|(x, y, start_time)| HitObject {
        pos: Pos2 {
            x: x as f32,
            y: y as f32,
        },
        start_time: start_time as f64,
        kind: HitObjectKind::Circle,
    })
    .collect();
    let timing_points = [TimingPoint {
        beat_len: 500.0,
        time: 0.0,
    }];

    let patterns =
        simple_beatmap_analyzer::detect_patterns(&objects, &timing_points, 4.0, 16.0, 110.0)
            .expect("four objects are enough for pattern detection");

    assert_eq!(patterns.total_notes, 4);
    assert_eq!(patterns.short_jump_notes, 4);
    assert_eq!(patterns.jump_notes_under_8, 4);
    assert_eq!(patterns.jump_notes_8_plus, 0);
    assert!((patterns.short_jump_pct_under_8() - 100.0).abs() < 0.01);
    assert!((patterns.long_jump_pct_8_plus() - 0.0).abs() < 0.01);
}

#[test]
fn one_third_snap_is_detected() {
    let patterns = analyze_file(fixture("one_third.osu"), &AnalyzeOptions::default())
        .unwrap()
        .patterns
        .unwrap();

    assert_eq!(patterns.total_notes, 6);
    assert_eq!(patterns.one_third_notes, 6);
    assert!((patterns.one_third_pct() - 100.0).abs() < 0.01);
    assert_eq!(patterns.stream_notes, 0);
    assert_eq!(patterns.jump_notes(), 0);
}

#[test]
fn non_standard_and_short_maps_have_no_patterns() {
    let taiko = analyze_file(fixture("taiko.osu"), &AnalyzeOptions::default()).unwrap();
    assert_eq!(taiko.metadata.mode, Mode::Taiko);
    assert!(taiko.patterns.is_none());
    assert!(taiko.difficulty.stars_nm > 0.0);

    let short = analyze_file(fixture("short.osu"), &AnalyzeOptions::default()).unwrap();
    assert_eq!(short.metadata.mode, Mode::Standard);
    assert!(short.patterns.is_none());
}

#[test]
fn analyze_bytes_matches_analyze_file() {
    let bytes = std::fs::read(fixture("standard_50.osu")).unwrap();
    let options = AnalyzeOptions::default();

    let from_path = analyze_file(fixture("standard_50.osu"), &options).unwrap();
    let from_bytes = analyze_bytes(&bytes, &options).unwrap();

    assert_eq!(from_path, from_bytes);
    assert_eq!(from_bytes.md5.len(), 32);
}

#[test]
fn invalid_options_are_rejected() {
    let options = AnalyzeOptions {
        accuracy: 101.0,
        ..AnalyzeOptions::default()
    };
    assert!(options.validate().is_err());

    let options = AnalyzeOptions {
        stream_distance: -5.0,
        ..AnalyzeOptions::default()
    };
    assert!(options.validate().is_err());

    let options = AnalyzeOptions {
        jump_distance: f32::NAN,
        ..AnalyzeOptions::default()
    };
    assert!(options.validate().is_err());
}

#[test]
fn csv_export_respects_columns_and_missing_patterns() {
    let standard = analyze_file(fixture("standard_50.osu"), &AnalyzeOptions::default()).unwrap();
    let taiko = analyze_file(fixture("taiko.osu"), &AnalyzeOptions::default()).unwrap();

    let files = vec![
        FileAnalysis {
            path: fixture("standard_50.osu"),
            analysis: standard,
        },
        FileAnalysis {
            path: fixture("taiko.osu"),
            analysis: taiko,
        },
    ];

    let columns = vec![
        Column::Title,
        Column::Stars,
        Column::Streams,
        Column::MapSetId,
    ];
    let mut buffer = Vec::new();
    write_csv(&mut buffer, &files, &columns).unwrap();
    let text = String::from_utf8(buffer).unwrap();
    let mut lines = text.lines();

    assert_eq!(lines.next(), Some("Title,Stars,Streams,MapSetID"));

    let standard_row: Vec<&str> = lines.next().unwrap().split(',').collect();
    assert_eq!(standard_row[0], "Fixture Song: The Rebirth");
    assert_eq!(standard_row[2], "40.00");
    assert_eq!(standard_row[3], "6789");

    let taiko_row: Vec<&str> = lines.next().unwrap().split(',').collect();
    assert_eq!(taiko_row[0], "Taiko Fixture");
    assert_eq!(taiko_row[2], ""); // non-standard maps have no pattern stats
}

#[test]
fn column_names_and_aliases_are_parsed() {
    let columns = parse_columns("title, SR, streams, jump_notes, MapSetID").unwrap();
    assert_eq!(
        columns,
        vec![
            Column::Title,
            Column::Stars,
            Column::Streams,
            Column::JumpNotes,
            Column::MapSetId
        ]
    );

    assert!(parse_columns("not-a-column").is_err());
    assert!(parse_columns("  ").is_err());
}

#[test]
fn cache_round_trip_and_option_isolation() {
    let dir = temp_dir("cache");
    let path = dir.join("cache.bin");
    let options = AnalyzeOptions::default();
    let analysis = analyze_file(fixture("standard_50.osu"), &options).unwrap();

    {
        let mut cache = Cache::empty(path.clone());
        cache.insert(&analysis, &options);
        assert_eq!(cache.len(), 1);
        cache.save().unwrap();
    }

    let mut cache = Cache::open(path.clone()).unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(&analysis.md5, &options).unwrap(), analysis);

    let changed = AnalyzeOptions {
        stream_distance: 32.0,
        ..options.clone()
    };
    assert!(cache.get(&analysis.md5, &changed).is_none());

    // The path is not part of the key: a moved/renamed file is still a hit.
    let moved = dir.join("moved.osu");
    std::fs::copy(fixture("standard_50.osu"), &moved).unwrap();
    let outcome = analyze_paths(std::slice::from_ref(&moved), &options, Some(&mut cache));
    assert_eq!(outcome.cached, 1);
    assert_eq!(outcome.files[0].path, moved);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scan_finds_and_analyzes_every_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let paths = find_beatmaps(&root).unwrap();

    assert_eq!(paths.len(), 5);
    assert!(paths.windows(2).all(|pair| pair[0] <= pair[1]));

    let options = AnalyzeOptions::default();
    let outcome = analyze_paths(&paths, &options, None);

    assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
    assert_eq!(outcome.files.len(), 5);
    assert_eq!(outcome.cached, 0);
    assert_eq!(outcome.analyzed(), 5);
    assert_eq!(outcome.total(), 5);
}
