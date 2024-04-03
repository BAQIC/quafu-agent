// Executes an OpenQASM program read from the input stream, repeatedly if the
// number of repetitions is passed as the first argument. If there is a second
// argument (i.e., argc == 3), then the final quantum state is displayed. If
// there are three or more arguments (i.e., argc  > 3), the projector onto the
// final state is displayed.
// Source: ./examples/qasm/qpp_qasm.cpp

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <ostream>
#include <qpp/classes/qcircuit.hpp>
#include <qpp/classes/qengine.hpp>
#include <qpp/functions.hpp>
#include <qpp/qasm/qasm.hpp>
#include <string>
#include <variant>
#include "argparse.hpp"
#include "qpp/qpp.h"

int main(int argc, char** argv) {
    argparse::ArgumentParser program("qpp-agent");
    program.add_argument("--file", "-f")
        .default_value("./example/bell.qasm")
        .help("OpenQASM file paht");
    program.add_argument("--shots", "-s")
        .default_value((unsigned int)0)
        .help("Simulation shots")
        .scan<'u', unsigned int>();
    program.add_argument("--simulator")
        .default_value("sv")
        .choices("sv", "dm")
        .help("The simulator which you will use, sv for statevector, dm for "
              "density matrix");
    program.add_argument("--output-file", "-o")
        .default_value("data")
        .help("The output file path which use to save the result");

    try {
        program.parse_args(argc, argv);
    } catch (const std::exception& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        return 1;
    }

    qpp::QCircuit qc =
        qpp::qasm::read_from_file(program.get<std::string>("--file"));
    std::ofstream stats_file(program.get<std::string>("--output-file") +
                             ".stats");
    std::ofstream state_file(program.get<std::string>("--output-file") +
                             ".state");
    auto shots = program.get<unsigned int>("--shots") == 0 ? 1: program.get<unsigned int>("--shots");

    if (program.get<std::string>("--simulator") == "sv") {
        qpp::QKetEngine q_sv_engine{qc};
        q_sv_engine.execute(shots);
        auto stats = q_sv_engine.get_stats().data();
        for (auto& [key, value] : stats) {
            for (auto& data : key) {
                stats_file << data << " ";
            }
            stats_file << value << std::endl;
        }
        for (auto& data : q_sv_engine.get_state()) {
            state_file << data.real() << " " << data.imag() << std::endl;
        }
    } else {
        qpp::QDensityEngine q_dm_engine{qc};
        q_dm_engine.execute(shots);
        auto stats = q_dm_engine.get_stats().data();
        for (auto& [key, value] : stats) {
            for (auto& data : key) {
                stats_file << data << " ";
            }
            stats_file << value << std::endl;
        }

        auto state = q_dm_engine.get_state();
        for (size_t i = 0, nrows = state.rows(), nclos = state.cols();
             i < nrows; i++) {
            for (size_t j = 0; j < nclos; j++) {
                state_file << state(i, j).real() << " " << state(i, j).imag()
                           << " ";
            }
            state_file << std::endl;
        }
    }
}
