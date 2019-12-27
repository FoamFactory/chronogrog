<img src="assets/logo.svg" width="200" />

# Chronogrog
A tool for creating brewing production schedules

## Description
`chronogrog` is a tool for creating [Gantt charts](https://en.wikipedia.org/wiki/Gantt_chart) for brewery production schedules. The input is a [BPD (beer production description file)](#beer-production-description-file-format), which is a JSON file in a specific format (defined below).

The system is designed to be used with [pla](https://github.com/jwir3/pla), a tool for creating Gantt charts from a descriptive file format.

## Roadmap
Currently, the chronogrog tool converts a file from a beer production description format into the necessary format for pla to generate a Gantt chart. Currently, the following features are supported:

  - Tracking of resources to support single usage of resources for a given point in time
  - Templates for brewing phases to eliminate duplicated data as much as possible within the beer production description file.

Within our [issues listing](https://github.com/foamfactory/chronogrog/issues) there are a number of issues targeting new feature development. Eventually, we want to completely incorporate the pla tool within chronogrog. The concept is to be able to go from a beer production description file directly to a styleable Gantt chart, in html or svg format.

## Beer Production Description File Format
